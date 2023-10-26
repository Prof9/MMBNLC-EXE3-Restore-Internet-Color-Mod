use ilhook::x64::{
    CallbackOption, HookFlags, HookPoint, HookType, Hooker, JmpToRetRoutine, Registers,
};
use object::{read::pe::PeFile64, Object, ObjectSection};
use std::os::raw::{c_int, c_void};
use wchar::wchz;
use winapi::{
    shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE},
    um::{libloaderapi::GetModuleHandleW, winnt::DLL_PROCESS_DETACH},
};

pub mod memsearch;

const FLAG_Z: u32 = 4;

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct GbaState {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,
    pub r13: u32,
    pub r14: u32,
    pub r15: u32,
    pub flags: u32,
    pub flags_enabled: u32,
    pub ram: *const u8,
    pub unk50: u32,
    pub unk54: u32,
    pub unk58: u32,
    pub unk5c: u32,
    pub ldmia_stmia_addr: u32,
    pub stack_size: u32,
    pub call_depth: u32,
}

impl GbaState {
    pub fn read_u8(&self, addr: u32) -> u8 {
        unsafe { *(self.ram.offset(addr.try_into().unwrap()) as *const u8) }
    }
    pub fn read_u16(&self, addr: u32) -> u16 {
        unsafe { *(self.ram.offset(addr.try_into().unwrap()) as *const u16) }
    }
    pub fn read_u32(&self, addr: u32) -> u32 {
        unsafe { *(self.ram.offset(addr.try_into().unwrap()) as *const u32) }
    }

    pub fn from_addr<'a>(addr: u64) -> &'a mut Self {
        unsafe { &mut *(addr as *mut Self) }
    }
}

type GbaFunc = extern "C" fn(*mut GbaState) -> u32;

static mut HOOKS: Vec<HookPoint> = Vec::new();

#[allow(non_upper_case_globals)]
static mut EXE3_SceFlagTest2: Option<GbaFunc> = None;

#[no_mangle]
pub extern "system" fn DllMain(_module: HINSTANCE, call_reason: DWORD, _reserved: LPVOID) -> BOOL {
    if call_reason == DLL_PROCESS_DETACH {
        unsafe { &mut HOOKS }.clear();
    }
    TRUE
}

fn hook_direct(addr: usize, func: JmpToRetRoutine, user_data: usize) {
    println!("Hooking {addr:#X}");
    let hooker = Hooker::new(
        addr,
        HookType::JmpToRet(func),
        CallbackOption::None,
        user_data,
        HookFlags::empty(),
    );
    let hook = unsafe { hooker.hook() };
    let hook = hook.expect(format!("Failed to hook {addr:#X}!").as_str());

    unsafe { &mut HOOKS }.push(hook);
}

#[no_mangle]
pub unsafe extern "C" fn luaopen_patch(_: c_void) -> c_int {
    let module = GetModuleHandleW(wchz!("MMBN_LC1.exe").as_ptr());
    let headers = std::slice::from_raw_parts(module as *const u8, 0x400);
    let Ok(pe_header) = PeFile64::parse(headers) else {
        eprintln!("Cannot parse module header from {module:#?}!");
        return 1;
    };
    let Some(text_section) = pe_header.section_by_name(".text") else {
        eprintln!("Cannot find .text section!");
        return 1;
    };
    let text_start = text_section.address() as usize;
    let text_size = text_section.size() as usize;
    println!("Found .text section @ {text_start:#X}, size {text_size:#X}");

    println!("Setting .text section writable...");
    if region::protect(
        text_start as *const u8,
        text_size,
        region::Protection::READ_WRITE_EXECUTE,
    ).is_err() {
        eprintln!("Cannot set .text section writable!");
        return 1;
    }

    // Find EXE3_SceFlagTest2
    println!("Searching for EXE3_SceFlagTest2...");
    let ptrs_sce_flag_test2 = memsearch::find_n_in(
        "48895C2410 48896C2418 4889742420 48894C2408 57 4154 4155 4156 4157 4883EC20 488BD9 488D0Dxxxxxxxx E8xxxxxxxx C7433CA1280000",
        text_start, text_size, 1
    );
    if ptrs_sce_flag_test2.is_err() || ptrs_sce_flag_test2.as_ref().unwrap().len() != 1 {
        eprintln!("Cannot find EXE3_SceFlagTest2!");
        return 1;
    }
    unsafe { EXE3_SceFlagTest2 = Some(std::mem::transmute(ptrs_sce_flag_test2.unwrap()[0])); }
    println!("Found EXE3_SceFlaTest2 @ {:#X?}", EXE3_SceFlagTest2.unwrap());

    // Find EXE3_St90ScrChgCheck
    println!("Searching for EXE3_St90ScrChgCheck...");
    let ptrs_st90_scr_chg_check = memsearch::find_n_in(
        "8B4340 C1E802 A801 7522|488D531C 488BCB",
        text_start, text_size, 2
    );
    if ptrs_st90_scr_chg_check.is_err() || ptrs_st90_scr_chg_check.as_ref().unwrap().len() != 2 {
        eprintln!("Cannot find EXE3_St90ScrChgCheck!");
        return 1;
    }

    // Install hooks
    for addr in ptrs_st90_scr_chg_check.unwrap().iter() {
        println!("Found EXE3_St90ScrChgCheck @ {addr:#X}");
        hook_direct(*addr, on_scr_chg_check, *addr);
    }
    
    println!("OK!");
    0
}

extern "win64" fn on_scr_chg_check(
    reg: *mut Registers,
    return_addr: usize,
    from_addr: usize,
) -> usize {
    // mov     r0,0xA
    // mov     r1,0x6
    // bl      scenario::SceFlagTest
    // beq     ...
    // -- HOOK --
    // mov     r0,r7
    // bl      scrchg::ScrChgTransSetAll

    let gba = unsafe { GbaState::from_addr((*reg).rbx) };

    // Check if game clear flag set
    // Same check as no panic music after beating game
    gba.r0 = 0xA00;
    unsafe { EXE3_SceFlagTest2.unwrap()(gba); }
    if (gba.flags & FLAG_Z) == 0 { // game clear flag not set
        // Skip switch to muted Internet colors
        return from_addr + 0x22
    } else {
        // Continue normally
        return return_addr
    }

    /*
    // Check if game clear star obtained
    let title_data = gba.read_u32(gba.r10 + 0x34);
    let stars = gba.read_u8(title_data + 0xA);
    let is_game_clear = (stars & 0x80) != 0;

    // Check if game in final state
    let story_data = gba.read_u32(gba.r10 + 0x8);
    let story_chapter = gba.read_u8(story_data + 0x6);
    let is_final_state = story_chapter == 0x79;

    if is_game_clear && is_final_state {
        // Skip switch to muted Internet colors
        from_addr + 0x22
    } else {
        // Continue normally
        return_addr
    }
    */
}
