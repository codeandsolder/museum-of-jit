use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use std::time::Instant;
#[derive(Clone,Copy)] struct Ins{op:u8,src:u8,dst:u8,imm:u8}
#[derive(Clone,Copy)] struct Program{ins:[Ins;8],uncond:bool}
const P:[Program;32]=[Program{ins:[Ins{op:3,src:8,dst:7,imm:40}, Ins{op:12,src:4,dst:6,imm:2}, Ins{op:6,src:6,dst:5,imm:54}, Ins{op:11,src:5,dst:3,imm:2}, Ins{op:1,src:3,dst:2,imm:33}, Ins{op:1,src:7,dst:0,imm:9}, Ins{op:0,src:2,dst:1,imm:9}, Ins{op:5,src:1,dst:4,imm:60}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:4,imm:20}, Ins{op:12,src:3,dst:5,imm:2}, Ins{op:11,src:5,dst:3,imm:3}, Ins{op:1,src:2,dst:7,imm:9}, Ins{op:1,src:7,dst:0,imm:1}, Ins{op:4,src:4,dst:2,imm:45}, Ins{op:1,src:6,dst:1,imm:1}, Ins{op:6,src:1,dst:6,imm:51}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:0,imm:10}, Ins{op:8,src:7,dst:2,imm:1}, Ins{op:1,src:4,dst:6,imm:33}, Ins{op:10,src:3,dst:5,imm:1}, Ins{op:0,src:0,dst:1,imm:33}, Ins{op:1,src:6,dst:4,imm:9}, Ins{op:4,src:1,dst:3,imm:24}, Ins{op:9,src:5,dst:7,imm:2}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:5,imm:24}, Ins{op:8,src:0,dst:1,imm:3}, Ins{op:10,src:7,dst:6,imm:1}, Ins{op:11,src:4,dst:3,imm:3}, Ins{op:4,src:5,dst:2,imm:37}, Ins{op:1,src:3,dst:4,imm:33}, Ins{op:2,src:2,dst:7,imm:9}, Ins{op:2,src:1,dst:0,imm:1}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:1,imm:49}, Ins{op:10,src:1,dst:0,imm:2}, Ins{op:10,src:6,dst:4,imm:1}, Ins{op:5,src:7,dst:5,imm:10}, Ins{op:9,src:5,dst:7,imm:2}, Ins{op:0,src:3,dst:6,imm:9}, Ins{op:0,src:0,dst:3,imm:1}, Ins{op:0,src:4,dst:2,imm:9}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:3,imm:63}, Ins{op:1,src:0,dst:5,imm:1}, Ins{op:9,src:4,dst:1,imm:3}, Ins{op:11,src:1,dst:4,imm:2}, Ins{op:12,src:3,dst:2,imm:2}, Ins{op:0,src:6,dst:7,imm:9}, Ins{op:1,src:7,dst:6,imm:33}, Ins{op:10,src:2,dst:0,imm:2}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:4,imm:59}, Ins{op:0,src:7,dst:3,imm:1}, Ins{op:10,src:3,dst:6,imm:2}, Ins{op:2,src:6,dst:0,imm:9}, Ins{op:4,src:0,dst:2,imm:11}, Ins{op:9,src:2,dst:5,imm:1}, Ins{op:1,src:1,dst:7,imm:9}, Ins{op:6,src:5,dst:1,imm:5}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:0,imm:57}, Ins{op:8,src:7,dst:4,imm:3}, Ins{op:12,src:4,dst:2,imm:2}, Ins{op:2,src:1,dst:6,imm:1}, Ins{op:11,src:5,dst:3,imm:2}, Ins{op:4,src:6,dst:5,imm:6}, Ins{op:0,src:3,dst:7,imm:1}, Ins{op:1,src:0,dst:1,imm:1}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:6,imm:35}, Ins{op:5,src:1,dst:2,imm:18}, Ins{op:2,src:6,dst:7,imm:1}, Ins{op:2,src:5,dst:3,imm:33}, Ins{op:8,src:4,dst:0,imm:3}, Ins{op:12,src:3,dst:1,imm:3}, Ins{op:0,src:0,dst:5,imm:9}, Ins{op:7,src:2,dst:4,imm:3}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:4,imm:34}, Ins{op:5,src:3,dst:1,imm:16}, Ins{op:7,src:7,dst:6,imm:1}, Ins{op:1,src:1,dst:7,imm:33}, Ins{op:1,src:0,dst:2,imm:33}, Ins{op:5,src:2,dst:0,imm:32}, Ins{op:8,src:6,dst:3,imm:1}, Ins{op:1,src:4,dst:5,imm:9}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:1,imm:1}, Ins{op:10,src:6,dst:7,imm:1}, Ins{op:1,src:3,dst:2,imm:1}, Ins{op:0,src:7,dst:3,imm:1}, Ins{op:1,src:4,dst:5,imm:33}, Ins{op:12,src:5,dst:4,imm:1}, Ins{op:4,src:1,dst:0,imm:9}, Ins{op:7,src:2,dst:6,imm:2}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:3,imm:17}, Ins{op:5,src:3,dst:0,imm:35}, Ins{op:10,src:1,dst:5,imm:1}, Ins{op:0,src:7,dst:4,imm:33}, Ins{op:6,src:4,dst:1,imm:7}, Ins{op:0,src:6,dst:2,imm:9}, Ins{op:1,src:5,dst:7,imm:33}, Ins{op:4,src:0,dst:6,imm:55}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:0,imm:21}, Ins{op:7,src:6,dst:5,imm:3}, Ins{op:11,src:5,dst:2,imm:2}, Ins{op:0,src:2,dst:7,imm:9}, Ins{op:7,src:7,dst:3,imm:1}, Ins{op:0,src:3,dst:1,imm:1}, Ins{op:0,src:0,dst:6,imm:1}, Ins{op:7,src:1,dst:4,imm:1}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:5,imm:41}, Ins{op:5,src:3,dst:6,imm:55}, Ins{op:10,src:6,dst:7,imm:1}, Ins{op:1,src:2,dst:4,imm:9}, Ins{op:6,src:0,dst:1,imm:36}, Ins{op:10,src:4,dst:2,imm:2}, Ins{op:0,src:5,dst:3,imm:1}, Ins{op:1,src:1,dst:0,imm:33}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:6,imm:19}, Ins{op:5,src:6,dst:3,imm:61}, Ins{op:2,src:2,dst:5,imm:1}, Ins{op:6,src:4,dst:7,imm:54}, Ins{op:10,src:5,dst:2,imm:2}, Ins{op:2,src:7,dst:4,imm:33}, Ins{op:0,src:0,dst:1,imm:1}, Ins{op:5,src:3,dst:0,imm:62}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:1,imm:50}, Ins{op:5,src:6,dst:4,imm:35}, Ins{op:0,src:5,dst:2,imm:1}, Ins{op:1,src:1,dst:3,imm:1}, Ins{op:8,src:3,dst:6,imm:1}, Ins{op:9,src:2,dst:5,imm:2}, Ins{op:0,src:7,dst:0,imm:33}, Ins{op:8,src:0,dst:7,imm:2}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:0,imm:31}, Ins{op:1,src:6,dst:5,imm:33}, Ins{op:9,src:1,dst:7,imm:1}, Ins{op:0,src:0,dst:3,imm:9}, Ins{op:5,src:2,dst:6,imm:36}, Ins{op:4,src:5,dst:4,imm:37}, Ins{op:0,src:3,dst:1,imm:1}, Ins{op:11,src:7,dst:2,imm:2}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:3,imm:58}, Ins{op:12,src:6,dst:7,imm:3}, Ins{op:1,src:7,dst:6,imm:33}, Ins{op:11,src:2,dst:5,imm:1}, Ins{op:10,src:0,dst:4,imm:1}, Ins{op:0,src:5,dst:0,imm:1}, Ins{op:7,src:4,dst:2,imm:1}, Ins{op:1,src:3,dst:1,imm:33}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:5,imm:34}, Ins{op:9,src:6,dst:7,imm:2}, Ins{op:6,src:7,dst:4,imm:1}, Ins{op:11,src:4,dst:0,imm:2}, Ins{op:1,src:0,dst:2,imm:9}, Ins{op:10,src:5,dst:1,imm:2}, Ins{op:0,src:2,dst:3,imm:33}, Ins{op:1,src:3,dst:6,imm:33}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:2,imm:11}, Ins{op:4,src:7,dst:6,imm:55}, Ins{op:7,src:3,dst:0,imm:3}, Ins{op:10,src:0,dst:7,imm:1}, Ins{op:2,src:2,dst:4,imm:1}, Ins{op:7,src:4,dst:3,imm:3}, Ins{op:1,src:5,dst:1,imm:9}, Ins{op:1,src:1,dst:5,imm:33}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:0,imm:24}, Ins{op:11,src:4,dst:2,imm:1}, Ins{op:2,src:3,dst:6,imm:1}, Ins{op:0,src:7,dst:1,imm:1}, Ins{op:11,src:2,dst:4,imm:1}, Ins{op:0,src:1,dst:3,imm:9}, Ins{op:4,src:6,dst:5,imm:42}, Ins{op:7,src:0,dst:7,imm:3}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:1,imm:20}, Ins{op:1,src:2,dst:7,imm:9}, Ins{op:6,src:6,dst:5,imm:44}, Ins{op:7,src:5,dst:2,imm:3}, Ins{op:7,src:1,dst:4,imm:2}, Ins{op:9,src:3,dst:0,imm:3}, Ins{op:0,src:0,dst:3,imm:33}, Ins{op:2,src:4,dst:6,imm:9}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:7,imm:22}, Ins{op:8,src:0,dst:5,imm:2}, Ins{op:11,src:4,dst:6,imm:2}, Ins{op:1,src:6,dst:2,imm:33}, Ins{op:0,src:1,dst:0,imm:9}, Ins{op:4,src:7,dst:3,imm:26}, Ins{op:5,src:3,dst:4,imm:1}, Ins{op:1,src:5,dst:1,imm:33}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:7,imm:59}, Ins{op:8,src:1,dst:0,imm:2}, Ins{op:12,src:2,dst:5,imm:1}, Ins{op:1,src:0,dst:1,imm:1}, Ins{op:0,src:3,dst:2,imm:9}, Ins{op:2,src:5,dst:3,imm:33}, Ins{op:7,src:7,dst:6,imm:2}, Ins{op:11,src:6,dst:4,imm:2}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:7,imm:21}, Ins{op:7,src:3,dst:6,imm:3}, Ins{op:1,src:6,dst:1,imm:33}, Ins{op:2,src:1,dst:5,imm:9}, Ins{op:11,src:7,dst:0,imm:3}, Ins{op:6,src:5,dst:4,imm:31}, Ins{op:1,src:4,dst:2,imm:9}, Ins{op:10,src:2,dst:3,imm:2}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:4,imm:33}, Ins{op:11,src:7,dst:3,imm:2}, Ins{op:11,src:3,dst:0,imm:3}, Ins{op:1,src:5,dst:1,imm:1}, Ins{op:9,src:4,dst:6,imm:3}, Ins{op:0,src:2,dst:7,imm:1}, Ins{op:1,src:1,dst:2,imm:9}, Ins{op:11,src:6,dst:5,imm:2}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:3,imm:4}, Ins{op:11,src:5,dst:4,imm:2}, Ins{op:11,src:4,dst:5,imm:3}, Ins{op:1,src:3,dst:7,imm:1}, Ins{op:1,src:1,dst:2,imm:9}, Ins{op:0,src:2,dst:1,imm:9}, Ins{op:11,src:6,dst:0,imm:2}, Ins{op:4,src:0,dst:6,imm:22}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:4,imm:3}, Ins{op:12,src:1,dst:6,imm:1}, Ins{op:0,src:4,dst:1,imm:1}, Ins{op:0,src:5,dst:0,imm:33}, Ins{op:6,src:0,dst:3,imm:19}, Ins{op:12,src:2,dst:7,imm:3}, Ins{op:9,src:6,dst:5,imm:1}, Ins{op:0,src:7,dst:2,imm:9}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:6,imm:3}, Ins{op:0,src:3,dst:0,imm:9}, Ins{op:2,src:7,dst:5,imm:33}, Ins{op:0,src:2,dst:3,imm:9}, Ins{op:12,src:0,dst:2,imm:3}, Ins{op:11,src:5,dst:1,imm:3}, Ins{op:9,src:6,dst:4,imm:2}, Ins{op:9,src:4,dst:7,imm:3}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:0,imm:29}, Ins{op:0,src:0,dst:1,imm:9}, Ins{op:1,src:5,dst:4,imm:9}, Ins{op:4,src:4,dst:5,imm:47}, Ins{op:5,src:2,dst:3,imm:13}, Ins{op:6,src:3,dst:6,imm:51}, Ins{op:1,src:7,dst:2,imm:33}, Ins{op:4,src:1,dst:7,imm:31}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:6,imm:52}, Ins{op:4,src:7,dst:3,imm:39}, Ins{op:6,src:3,dst:4,imm:56}, Ins{op:9,src:4,dst:5,imm:3}, Ins{op:0,src:5,dst:2,imm:1}, Ins{op:12,src:6,dst:0,imm:3}, Ins{op:2,src:2,dst:1,imm:33}, Ins{op:0,src:1,dst:7,imm:1}],uncond:false},
Program{ins:[Ins{op:3,src:8,dst:7,imm:5}, Ins{op:10,src:1,dst:6,imm:2}, Ins{op:0,src:5,dst:4,imm:9}, Ins{op:0,src:0,dst:2,imm:1}, Ins{op:7,src:2,dst:5,imm:3}, Ins{op:4,src:4,dst:0,imm:62}, Ins{op:2,src:6,dst:1,imm:1}, Ins{op:9,src:7,dst:3,imm:2}],uncond:true}];
const K0:u64=1663540288323457296; const K1:u64=2242261671028070680;
#[inline(always)] fn sip(v0:&mut u64,v1:&mut u64,v2:&mut u64,v3:&mut u64){*v0=v0.wrapping_add(*v1);*v2=v2.wrapping_add(*v3);*v1=v1.rotate_left(13);*v3=v3.rotate_left(16);*v1^=*v0;*v3^=*v2;*v0=v0.rotate_left(32);*v2=v2.wrapping_add(*v1);*v0=v0.wrapping_add(*v3);*v1=v1.rotate_left(17);*v3=v3.rotate_left(21);*v1^=*v2;*v3^=*v0;*v2=v2.rotate_left(32);}
#[inline(always)] fn init(salt:u64)->[u64;9]{let(mut v0,mut v1,mut v2,mut v3)=(0x736f6d6570736575^K0,0x646f72616e646f6d^K1,0x6c7967656e657261^K0,0x7465646279746573^K1);v3^=salt;sip(&mut v0,&mut v1,&mut v2,&mut v3);v0^=salt;v2^=0xbb;for _ in 0..3{sip(&mut v0,&mut v1,&mut v2,&mut v3)}let(r0,r1,r2,r3)=(v3,v2,v1,v0);v0^=K0;v1^=K1;v2^=K0;v3^=K1;for _ in 0..4{sip(&mut v0,&mut v1,&mut v2,&mut v3)}let(r4,r5,r6,r7)=(v3,v2,v1,v0);let r8=((r0^r4)&!7)|3;[r0,r1,r2,r3,r4,r5,r6,r7,r8]}
#[inline(always)] fn finish(r:&[u64;9])->u64{let(mut a,mut b,mut c,mut d)=(r[0],r[1],r[2],r[3]);sip(&mut a,&mut b,&mut c,&mut d);let(mut e,mut f,mut g,mut h)=(r[4],r[5],r[6],r[7]);sip(&mut e,&mut f,&mut g,&mut h);d^h^r[8]}
fn emit_sip(ops:&mut dynasmrt::x64::Assembler,a:u8,b:u8,c:u8,d:u8){dynasm!(ops;
 add Rq(a),Rq(b); add Rq(c),Rq(d); rol Rq(b),13; rol Rq(d),16; xor Rq(b),Rq(a); xor Rq(d),Rq(c); rol Rq(a),32;
 add Rq(c),Rq(b); add Rq(a),Rq(d); rol Rq(b),17; rol Rq(d),21; xor Rq(b),Rq(c); xor Rq(d),Rq(a); rol Rq(c),32
);}
struct Jit{buf:dynasmrt::ExecutableBuffer,entry:*const u8}
impl Jit{fn compile()->Self{let mut ops=dynasmrt::x64::Assembler::new().unwrap();
 dynasm!(ops ; .arch x64 ; push rbx ; push r12 ; push r13 ; push r14 ; push r15
  ; mov r8,QWORD (0x736f6d6570736575u64^K0) as i64
  ; mov r9,QWORD (0x646f72616e646f6du64^K1) as i64
  ; mov r10,QWORD (0x6c7967656e657261u64^K0) as i64
  ; mov r11,QWORD (0x7465646279746573u64^K1) as i64
  ; xor r11,rdi);
 emit_sip(&mut ops,8,9,10,11);
 dynasm!(ops; xor r8,rdi; xor r10,0xbb);
 for _ in 0..3{emit_sip(&mut ops,8,9,10,11)}
 dynasm!(ops; mov r12,r11; mov r13,r10; mov r14,r9; mov r15,r8
  ; mov rax,QWORD K0 as i64; xor r8,rax; xor r10,rax
  ; mov rax,QWORD K1 as i64; xor r9,rax; xor r11,rax);
 for _ in 0..4{emit_sip(&mut ops,8,9,10,11)}
 dynasm!(ops; mov rax,r11; mov rcx,r10; mov rdx,r9; mov rsi,r8
  ; mov r8,r12; mov r9,r13; mov r10,r14; mov r11,r15
  ; mov r12,rax; mov r13,rcx; mov r14,rdx; mov r15,rsi
  ; mov rbx,r8; xor rbx,r12; and rbx,-8; or rbx,3; push rbx);
 emit_phase(&mut ops,false);emit_phase(&mut ops,true);
 dynasm!(ops; add rsp,16392);
 emit_sip(&mut ops,8,9,10,11);emit_sip(&mut ops,12,13,14,15);
 dynasm!(ops; mov rax,r11; xor rax,r15; xor rax,rbx; pop r15; pop r14; pop r13; pop r12; pop rbx; ret);
 let buf=ops.finalize().unwrap();let entry=buf.ptr(dynasmrt::AssemblyOffset(0));Self{buf,entry}}
 #[inline(always)] unsafe fn hash(&self,n:u64)->u64{let f:unsafe extern "sysv64" fn(u64)->u64=unsafe{std::mem::transmute(self.entry)};unsafe{f(n)}}}
#[inline] fn xr(r:u8)->u8{if r<8{8+r}else{3}}
fn source(ops:&mut dynasmrt::x64::Assembler,src:u8){let s=xr(src);dynasm!(ops ; mov eax,Rd(s) ; and eax,16383);}
fn emit_op(ops:&mut dynasmrt::x64::Assembler,x:Ins,mem:bool){let d=xr(x.dst);let s=xr(x.src);let im=x.imm as i8;match x.op{0=>{dynasm!(ops; or Rq(d),x.imm as i32);if mem{source(ops,x.src);dynasm!(ops; imul Rq(d),QWORD [rsp+rax])}else{dynasm!(ops; imul Rq(d),Rq(s))}},1=>{dynasm!(ops; xor Rq(d),x.imm as i32);if mem{source(ops,x.src);dynasm!(ops; imul Rq(d),QWORD [rsp+rax])}else{dynasm!(ops; imul Rq(d),Rq(s))}},2=>{dynasm!(ops; add Rq(d),x.imm as i32);if mem{source(ops,x.src);dynasm!(ops; imul Rq(d),QWORD [rsp+rax])}else{dynasm!(ops; imul Rq(d),Rq(s))}},3=>dynasm!(ops; imul Rq(d),rbx; ror Rq(d),im),4..=6=>{dynasm!(ops; ror Rq(d),im);if mem{source(ops,x.src);match x.op{4=>dynasm!(ops; xor Rq(d),QWORD [rsp+rax]),5=>dynasm!(ops; add Rq(d),QWORD [rsp+rax]),_=>dynasm!(ops; sub Rq(d),QWORD [rsp+rax])}}else{match x.op{4=>dynasm!(ops; xor Rq(d),Rq(s)),5=>dynasm!(ops; add Rq(d),Rq(s)),_=>dynasm!(ops; sub Rq(d),Rq(s))}}},7..=9=>{dynasm!(ops; sar Rq(d),im);if mem{source(ops,x.src);match x.op{7=>dynasm!(ops; xor Rq(d),QWORD [rsp+rax]),8=>dynasm!(ops; add Rq(d),QWORD [rsp+rax]),_=>dynasm!(ops; sub Rq(d),QWORD [rsp+rax])}}else{match x.op{7=>dynasm!(ops; xor Rq(d),Rq(s)),8=>dynasm!(ops; add Rq(d),Rq(s)),_=>dynasm!(ops; sub Rq(d),Rq(s))}}},10..=12=>{dynasm!(ops; shr Rq(d),im);if mem{source(ops,x.src);match x.op{10=>dynasm!(ops; xor Rq(d),QWORD [rsp+rax]),11=>dynasm!(ops; add Rq(d),QWORD [rsp+rax]),_=>dynasm!(ops; sub Rq(d),QWORD [rsp+rax])}}else{match x.op{10=>dynasm!(ops; xor Rq(d),Rq(s)),11=>dynasm!(ops; add Rq(d),Rq(s)),_=>dynasm!(ops; sub Rq(d),Rq(s))}}},_=>unreachable!()}}
fn emit_phase(ops:&mut dynasmrt::x64::Assembler,mem:bool){let rep=ops.new_dynamic_label();dynasm!(ops; mov esi,4 ; =>rep ; mov edx,32);for p in P{let start=ops.new_dynamic_label();let done=ops.new_dynamic_label();dynasm!(ops; =>start);if !mem{dynasm!(ops; push r8 ; push r9 ; push r10 ; push r11 ; push r12 ; push r13 ; push r14 ; push r15);}for x in p.ins{emit_op(ops,x,mem)}let bf=xr(p.ins[0].dst);dynasm!(ops; test edx,edx ; jz =>done);if !p.uncond{dynasm!(ops; test Rd(bf),32 ; jnz =>done);}dynasm!(ops; dec edx ; jmp =>start ; =>done);}dynasm!(ops; dec esi ; jnz =>rep);}
fn bench_count(name:&str,default:u64,allow_zero:bool)->u64{match std::env::var(name){Ok(raw)=>{let n=raw.parse::<u64>().unwrap_or_else(|_|panic!("{name} must be a non-negative integer, got {raw:?}"));assert!(allow_zero||n>0,"{name} must be > 0");n},Err(std::env::VarError::NotPresent)=>default,Err(err)=>panic!("cannot read {name}: {err}")}}
fn main(){let warmup=bench_count("MUSEUM_WARMUP",30000,true);let n=bench_count("MUSEUM_ITERATIONS",300000,false);let samples=bench_count("MUSEUM_SAMPLES",7,false);let t=Instant::now();let j=Jit::compile();let build=t.elapsed();let c=unsafe{j.hash(123456)};println!("BUILD_US {:.3} CODE {} CHECK {:016x}",build.as_secs_f64()*1e6,j.buf.len(),c);for i in 0..warmup{std::hint::black_box(unsafe{j.hash(i)});}for _ in 0..samples{let mut sink=0u64;let t=Instant::now();for i in 0..n{sink^=unsafe{j.hash(i)}}let sec=t.elapsed().as_secs_f64();println!("RATE {:.3} sink={:016x}",n as f64/sec,sink);}}
