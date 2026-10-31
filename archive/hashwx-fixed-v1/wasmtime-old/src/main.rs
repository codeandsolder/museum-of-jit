use challenge_hashwx::HashWx;
use std::time::Instant;
const N:u64=1_048_576;
fn main(){
 for sample in 0..3u8 {
   let mut seed=[0u8;32]; for(i,b)in seed.iter_mut().enumerate(){*b=sample.wrapping_mul(37).wrapping_add((i as u8).wrapping_mul(17)).wrapping_add(0x5a);}
   let mut h=HashWx::new().unwrap();
   let all=Instant::now(); h.make(&seed).unwrap(); let exec=Instant::now(); let mut sink=0u64;
   for n in 0..N { sink ^= h.hash(n).unwrap(); }
   let exec_s=exec.elapsed().as_secs_f64(); let all_s=all.elapsed().as_secs_f64();
   println!("sample={} exec_hps={:.3} including_make_hps={:.3} exec_s={:.6} all_s={:.6} sink={:016x}",sample,N as f64/exec_s,N as f64/all_s,exec_s,all_s,sink);
 }
}
