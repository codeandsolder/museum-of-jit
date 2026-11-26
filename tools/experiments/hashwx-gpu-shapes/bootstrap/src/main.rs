//! On-device CUDA architecture bootstrap: one ptxas compile, then extract native
//! SASS instructions and ELF kernel metadata without hardcoding file offsets.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;
use std::process::Command;

fn fail(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message.into())
}
fn u16_at(bytes: &[u8], at: usize) -> io::Result<u16> {
    Ok(u16::from_le_bytes(
        bytes
            .get(at..at + 2)
            .ok_or_else(|| fail("u16 bounds"))?
            .try_into()
            .map_err(|_| fail("u16 size"))?,
    ))
}
fn u32_at(bytes: &[u8], at: usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(at..at + 4)
            .ok_or_else(|| fail("u32 bounds"))?
            .try_into()
            .map_err(|_| fail("u32 size"))?,
    ))
}
fn u64_at(bytes: &[u8], at: usize) -> io::Result<u64> {
    Ok(u64::from_le_bytes(
        bytes
            .get(at..at + 8)
            .ok_or_else(|| fail("u64 bounds"))?
            .try_into()
            .map_err(|_| fail("u64 size"))?,
    ))
}
fn checked(value: u64) -> io::Result<usize> {
    usize::try_from(value).map_err(|_| fail("ELF offset overflow"))
}
#[derive(Clone)]
struct Section {
    offset: usize,
    len: usize,
    registers: u32,
}
struct Calibration {
    text: Section,
    info: Section,
    exits: Vec<u32>,
    exit_array_at: usize,
}
fn elf_sections(bytes: &[u8]) -> io::Result<BTreeMap<String, Section>> {
    if bytes.get(..6) != Some(&b"\x7fELF\x02\x01"[..]) {
        return Err(fail("expected little-endian ELF64"));
    }
    let table = checked(u64_at(bytes, 0x28)?)?;
    let entry_size = usize::from(u16_at(bytes, 0x3a)?);
    let count = usize::from(u16_at(bytes, 0x3c)?);
    let strings_index = usize::from(u16_at(bytes, 0x3e)?);
    if entry_size < 64 || count == 0 || strings_index >= count {
        return Err(fail("invalid ELF section table"));
    }
    let end = table
        .checked_add(
            entry_size
                .checked_mul(count)
                .ok_or_else(|| fail("section table overflow"))?,
        )
        .ok_or_else(|| fail("section table overflow"))?;
    if end > bytes.len() {
        return Err(fail("section table out of range"));
    }
    let str_at = table + strings_index * entry_size;
    let strings_offset = checked(u64_at(bytes, str_at + 24)?)?;
    let strings_len = checked(u64_at(bytes, str_at + 32)?)?;
    let strings = bytes
        .get(
            strings_offset
                ..strings_offset
                    .checked_add(strings_len)
                    .ok_or_else(|| fail("shstrtab overflow"))?,
        )
        .ok_or_else(|| fail("shstrtab outside file"))?;
    let mut sections = BTreeMap::new();
    for i in 0..count {
        let entry = table + i * entry_size;
        let name_offset =
            usize::try_from(u32_at(bytes, entry)?).map_err(|_| fail("name index overflow"))?;
        let raw = strings
            .get(name_offset..)
            .ok_or_else(|| fail("name outside string table"))?;
        let nul = raw
            .iter()
            .position(|c| *c == 0)
            .ok_or_else(|| fail("unterminated section name"))?;
        let name = std::str::from_utf8(&raw[..nul])
            .map_err(|_| fail("non-UTF8 section name"))?
            .to_owned();
        let offset = checked(u64_at(bytes, entry + 24)?)?;
        let len = checked(u64_at(bytes, entry + 32)?)?;
        if u32_at(bytes, entry + 4)? != 8
            && bytes
                .get(
                    offset
                        ..offset
                            .checked_add(len)
                            .ok_or_else(|| fail("section length overflow"))?,
                )
                .is_none()
        {
            return Err(fail(format!("section {name} out of file")));
        }
        sections.insert(
            name,
            Section {
                offset,
                len,
                registers: u32_at(bytes, entry + 44)? >> 24,
            },
        );
    }
    Ok(sections)
}
fn parse_cubin(bytes: &[u8], kernel: &str) -> io::Result<Calibration> {
    let sections = elf_sections(bytes)?;
    let text = sections
        .get(&format!(".text.{kernel}"))
        .ok_or_else(|| fail("missing kernel SASS text"))?
        .clone();
    let info = sections
        .get(&format!(".nv.info.{kernel}"))
        .ok_or_else(|| fail("missing kernel nv.info"))?
        .clone();
    if text.len == 0 || text.len % 16 != 0 {
        return Err(fail("unsupported SASS instruction stride"));
    }
    let attrs = &bytes[info.offset..info.offset + info.len];
    let mut exit = None;
    for i in 0..attrs.len().saturating_sub(3) {
        if attrs[i..i + 2] == [4, 0x1c] && exit.replace(i).is_some() {
            return Err(fail("duplicate EIATTR_EXIT_INSTR_OFFSETS"));
        }
    }
    let record = exit.ok_or_else(|| fail("missing EIATTR_EXIT_INSTR_OFFSETS"))?;
    let count = usize::from(u16_at(attrs, record + 2)?);
    if count == 0 || count % 4 != 0 || record + 4 + count > attrs.len() {
        return Err(fail("bad EXIT offsets record"));
    }
    let exits = (record + 4..record + 4 + count)
        .step_by(4)
        .map(|i| u32_at(attrs, i))
        .collect::<io::Result<Vec<_>>>()?;
    if exits
        .iter()
        .any(|&i| !usize::try_from(i).is_ok_and(|v| v < text.len && v % 16 == 0))
    {
        return Err(fail("EXIT metadata outside kernel text"));
    }
    let exit_array_at = info.offset + record + 4;
    Ok(Calibration {
        text,
        info,
        exits,
        exit_array_at,
    })
}
fn run_cmd(program: &Path, args: &[&str]) -> io::Result<String> {
    let output = Command::new(program).args(args).output()?;
    if !output.status.success() {
        return Err(fail(format!(
            "{}: {}",
            program.display(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout).map_err(|_| fail("non-UTF8 compiler output"))
}
fn auto_arch() -> io::Result<String> {
    let text = ["nvidia-smi", "/usr/lib/wsl/lib/nvidia-smi"]
        .into_iter()
        .find_map(|binary| {
            run_cmd(
                Path::new(binary),
                &["--query-gpu=compute_cap", "--format=csv,noheader,nounits"],
            )
            .ok()
        })
        .ok_or_else(|| fail("unable to query CUDA compute capability"))?;
    let cap = text
        .lines()
        .next()
        .ok_or_else(|| fail("GPU capability missing"))?;
    let digits = cap.chars().filter(char::is_ascii_digit).collect::<String>();
    if !(2..=3).contains(&digits.len()) {
        return Err(fail("invalid GPU compute capability"));
    }
    Ok(format!("sm_{digits}"))
}
fn samples(disasm: &str, text: &[u8]) -> String {
    let mut ops = BTreeMap::<String, Vec<(usize, String)>>::new();
    for line in disasm.lines() {
        let line = line.trim();
        let Some(tail) = line.strip_prefix("/*") else {
            continue;
        };
        let Some((offset, source)) = tail.split_once("*/") else {
            continue;
        };
        let Ok(pos) = usize::from_str_radix(offset.trim(), 16) else {
            continue;
        };
        if pos + 16 > text.len() || !source.contains(';') {
            continue;
        }
        let mut tokens = source.split_ascii_whitespace();
        let first = tokens.next().unwrap_or_default();
        let op = if first.starts_with('@') {
            tokens.next().unwrap_or_default()
        } else {
            first
        };
        let name = op.split('.').next().unwrap_or_default();
        if !name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            || name.is_empty()
        {
            continue;
        }
        let list = ops.entry(name.to_owned()).or_default();
        if list.len() >= 8 {
            continue;
        }
        let word =
            text[pos..pos + 16]
                .iter()
                .rev()
                .fold(String::with_capacity(32), |mut acc, byte| {
                    let _ = write!(acc, "{byte:02x}");
                    acc
                });
        list.push((pos, word));
    }
    let mut output =
        String::from("# Real per-arch opcode samples; no general operand encoder implied\n");
    for (name, list) in ops {
        for (offset, word) in list {
            let _ = writeln!(output, "{name}\t0x{offset:06x}\t0x{word}");
        }
    }
    output
}
fn run() -> io::Result<()> {
    let mut args = std::env::args().skip(1);
    let arch = args
        .next()
        .ok_or_else(|| fail("usage: bootstrap <sm_NN|auto> <source.ptx> <output_dir>"))?;
    let source = args.next().ok_or_else(|| fail("missing source.ptx"))?;
    let dest = args.next().ok_or_else(|| fail("missing output_dir"))?;
    if args.next().is_some() {
        return Err(fail("too many arguments"));
    }
    let arch = if arch == "auto" { auto_arch()? } else { arch };
    if !arch.starts_with("sm_") || !arch[3..].bytes().all(|b| b.is_ascii_digit()) {
        return Err(fail("invalid architecture"));
    }
    let ptxas = Path::new("/usr/local/cuda/bin/ptxas");
    let disasm = Path::new("/usr/local/cuda/bin/nvdisasm");
    let version = run_cmd(ptxas, &["--version"])?;
    let digest = run_cmd(Path::new("sha256sum"), &[&source])?;
    let stamp = format!("{arch}\n{version}\n{digest}\n");
    let dest = Path::new(&dest);
    fs::create_dir_all(dest)?;
    if fs::read_to_string(dest.join("stamp.txt")).is_ok_and(|v| v == stamp)
        && dest.join("reference.cubin").exists()
        && dest.join("manifest.txt").exists()
    {
        println!("cache_hit=true arch={arch}");
        return Ok(());
    }
    let temporary = dest.join(format!(".build-{}", std::process::id()));
    fs::create_dir_all(&temporary)?;
    let original = fs::read_to_string(&source)?;
    let target = original
        .lines()
        .find(|line| line.trim_start().starts_with(".target sm_"))
        .ok_or_else(|| fail("reference PTX lacks native CUDA target"))?;
    let transformed = original.replacen(target, &format!(".target {arch}"), 1);
    let reference_ptx = temporary.join("reference.ptx");
    fs::write(&reference_ptx, transformed)?;
    let cubin = temporary.join("reference.cubin");
    let cubin_path = cubin.to_str().ok_or_else(|| fail("non-UTF8 path"))?;
    run_cmd(
        ptxas,
        &[
            "-O3",
            &format!("-arch={arch}"),
            reference_ptx
                .to_str()
                .ok_or_else(|| fail("non-UTF8 PTX path"))?,
            "-o",
            cubin_path,
        ],
    )?;
    let bytes = fs::read(&cubin)?;
    let info = parse_cubin(&bytes, "hash_batch")?;
    let text = &bytes[info.text.offset..info.text.offset + info.text.len];
    fs::write(temporary.join("text.bin"), text)?;
    let rendered = run_cmd(disasm, &["-hex", "-c", cubin_path])?;
    fs::write(
        temporary.join("opcode-samples.tsv"),
        samples(&rendered, text),
    )?;
    fs::write(temporary.join("disassembly.sass"), rendered)?;
    let mut manifest = format!(
        "architecture={arch}\ntext_offset={}\ntext_bytes={}\ninfo_offset={}\ninfo_bytes={}\nexit_array_file_offset={}\n",
        info.text.offset, info.text.len, info.info.offset, info.info.len, info.exit_array_at
    );
    for exit in &info.exits {
        let _ = writeln!(manifest, "exit_text_offset={exit}");
    }
    fs::write(temporary.join("manifest.txt"), manifest)?;
    fs::write(temporary.join("stamp.txt"), stamp)?;
    for file in [
        "reference.cubin",
        "text.bin",
        "opcode-samples.tsv",
        "disassembly.sass",
        "manifest.txt",
        "stamp.txt",
        "reference.ptx",
    ] {
        fs::rename(temporary.join(file), dest.join(file))?;
    }
    fs::remove_dir(&temporary)?;
    println!(
        "cache_hit=false arch={arch} text_bytes={} exits={:?}",
        info.text.len, info.exits
    );
    Ok(())
}
// Native binary transplant is the minimal executable validation of our
// automatically discovered CUBIN/SASS patch locations. The input machine
// instructions must already be valid for the destination architecture and ABI.
fn transplant(shell: &[u8], donor: &[u8]) -> io::Result<Vec<u8>> {
    if u32_at(shell, 0x30)? != u32_at(donor, 0x30)? {
        return Err(fail(
            "cannot transplant SASS between different GPU architectures",
        ));
    }
    let shell_layout = parse_cubin(shell, "hash_batch")?;
    let donor_layout = parse_cubin(donor, "hash_batch")?;
    if donor_layout.text.len > shell_layout.text.len {
        return Err(fail("destination native instruction section too small"));
    }
    if donor_layout.text.registers > shell_layout.text.registers {
        return Err(fail("destination CUBIN allocates too few registers"));
    }
    if donor_layout.exits.len() != shell_layout.exits.len() {
        return Err(fail("different number of native kernel exit sites"));
    }
    let mut shell_info =
        shell[shell_layout.info.offset..shell_layout.info.offset + shell_layout.info.len].to_vec();
    let mut donor_info =
        donor[donor_layout.info.offset..donor_layout.info.offset + donor_layout.info.len].to_vec();
    let shell_exits = shell_layout.exit_array_at - shell_layout.info.offset;
    let donor_exits = donor_layout.exit_array_at - donor_layout.info.offset;
    for (idx, _) in shell_layout.exits.iter().enumerate() {
        shell_info[shell_exits + idx * 4..shell_exits + idx * 4 + 4].fill(0);
        donor_info[donor_exits + idx * 4..donor_exits + idx * 4 + 4].fill(0);
    }
    let zero_crs = [4_u8, 0x1e, 4, 0, 0, 0, 0, 0];
    for attributes in [&mut shell_info, &mut donor_info] {
        if let Some(at) = attributes.windows(8).position(|window| window == zero_crs) {
            attributes.drain(at..at + 8);
        }
    }
    if shell_info != donor_info {
        return Err(fail(
            "different non-exit CUDA ABI metadata; refusing native transplant",
        ));
    }
    let mut output = shell.to_vec();
    let source = &donor[donor_layout.text.offset..donor_layout.text.offset + donor_layout.text.len];
    let start = shell_layout.text.offset;
    output[start..start + source.len()].copy_from_slice(source);
    for (idx, offset) in donor_layout.exits.iter().enumerate() {
        let start = shell_layout.exit_array_at + idx * 4;
        output[start..start + 4].copy_from_slice(&offset.to_le_bytes());
    }
    Ok(output)
}

fn patch_command() -> io::Result<()> {
    let mut args = std::env::args().skip(2);
    let shell = args.next().ok_or_else(|| fail("patch needs SHELL_CUBIN"))?;
    let donor = args.next().ok_or_else(|| fail("patch needs DONOR_CUBIN"))?;
    let output = args
        .next()
        .ok_or_else(|| fail("patch needs OUTPUT_CUBIN"))?;
    if args.next().is_some() {
        return Err(fail("unexpected arguments to patch"));
    }
    let bytes = transplant(&fs::read(shell)?, &fs::read(donor)?)?;
    fs::write(&output, bytes)?;
    println!("native_patch_validated=true output={output}");
    Ok(())
}

fn main() {
    let task = if std::env::args().nth(1).as_deref() == Some("patch") {
        patch_command()
    } else {
        run()
    };
    if let Err(e) = task {
        eprintln!("bootstrap: {e}");
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_elf_header_and_bounds() {
        assert!(elf_sections(b"").is_err());
        assert!(parse_cubin(&[0; 64], "hash_batch").is_err());
    }
    #[test]
    fn extracts_opcode_text() {
        let mut bytes = [0; 32];
        bytes[16..].fill(0xaa);
        let output = samples(
            "/*0000*/ IMAD R1,R2,R3,R4 ;\n/*0010*/ @P0 LDS.64 R3,[R5] ;\n",
            &bytes,
        );
        assert!(output.contains("IMAD\t0x000000"));
        assert!(output.contains("LDS\t0x000010"));
    }
}
