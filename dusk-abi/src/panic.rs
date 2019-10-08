use core::fmt::Write;
use core::panic::PanicInfo;

use crate::bufwriter::BufWriter;
use crate::PANIC_BUFFER_SIZE;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut buffer = [0u8; PANIC_BUFFER_SIZE];
    let mut bw = BufWriter::new(&mut buffer);

    if let Some(msg) = info.message() {
        write!(bw, "{}", msg).unwrap();
    }
    let len = bw.ofs() as i32;
    unsafe { super::external::panic(&buffer[0], len) }
}
