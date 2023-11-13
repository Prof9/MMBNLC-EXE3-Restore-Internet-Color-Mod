pub mod memsearch;
pub mod mmbnlc;

use crate::mmbnlc::*;
use mlua::prelude::*;

static mut HOOKS: Vec<ilhook::x64::HookPoint> = Vec::new();

#[allow(non_upper_case_globals)]
static EXE3_SceFlagTest2: std::sync::OnceLock<GBAFunc> = std::sync::OnceLock::new();

fn hook_direct(addr: usize, func: ilhook::x64::JmpToRetRoutine, user_data: usize) {
    let hooker = ilhook::x64::Hooker::new(
        addr,
        ilhook::x64::HookType::JmpToRet(func),
        ilhook::x64::CallbackOption::None,
        user_data,
        ilhook::x64::HookFlags::empty(),
    );
    let hook = unsafe { hooker.hook() };
    let hook = hook.expect(format!("Failed to hook {addr:#X}!").as_str());

    unsafe { &mut HOOKS }.push(hook);
}

#[mlua::lua_module]
fn patch(lua: &Lua) -> LuaResult<LuaValue> {
    let text_section = lua
        .globals()
        .get::<_, LuaTable>("chaudloader")?
        .get::<_, LuaTable>("GAME_ENV")?
        .get::<_, LuaTable>("sections")?
        .get::<_, LuaTable>("text")?;
    let text_address = text_section.get::<_, LuaInteger>("address")? as usize;
    let text_size = text_section.get::<_, LuaInteger>("size")? as usize;

    // Find EXE3_SceFlagTest2
    println!("Searching for EXE3_SceFlagTest2...");
    let ptrs_sce_flag_test2 = memsearch::find_n_in(
        "48895C2410 48896C2418 4889742420 48894C2408 57 4154 4155 4156 4157 4883EC20 488BD9 488D0Dxxxxxxxx E8xxxxxxxx C7433CA1280000",
        text_address, text_size, 1
    );
    if ptrs_sce_flag_test2.is_err() || ptrs_sce_flag_test2.as_ref().unwrap().len() != 1 {
        panic!("Cannot find EXE3_SceFlagTest2!");
    }
    EXE3_SceFlagTest2
        .set(unsafe { std::mem::transmute(ptrs_sce_flag_test2.unwrap()[0]) })
        .unwrap();
    println!("Found EXE3_SceFlagTest2 @ {:#X}", EXE3_SceFlagTest2.get().unwrap() as *const GBAFunc as usize);

    // Find EXE3_St90ScrChgCheck
    println!("Searching for EXE3_St90ScrChgCheck...");
    let ptrs_st90_scr_chg_check = memsearch::find_n_in(
        "8B4340 C1E802 A801 7522|488D531C 488BCB",
        text_address,
        text_size,
        2,
    );
    if ptrs_st90_scr_chg_check.is_err() || ptrs_st90_scr_chg_check.as_ref().unwrap().len() != 2 {
        panic!("Cannot find EXE3_St90ScrChgCheck!");
    }

    // Install hooks
    for addr in ptrs_st90_scr_chg_check.unwrap().iter() {
        println!("Hooking EXE3_St90ScrChgCheck @ {addr:#X}");
        hook_direct(*addr, on_scr_chg_check, *addr);
    }

    Ok(LuaValue::Nil)
}

unsafe extern "win64" fn on_scr_chg_check(
    reg: *mut ilhook::x64::Registers,
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

    let gba = GBAState::from_addr((*reg).rbx);

    // Check if game clear flag set
    // Same check as no panic music after beating game
    gba.r0 = 0xA00;
    EXE3_SceFlagTest2.get().unwrap()(gba);
    if !gba.flags.contains(CPUFlags::Z) {
        // game clear flag not set
        // Skip switch to muted Internet colors
        from_addr + 0x22
    } else {
        // Continue normally
        return_addr
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
