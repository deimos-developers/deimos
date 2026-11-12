//! Local player and credit operations.

use core::ffi::c_void;

/// `HouseClass::CurrentPlayer`, 0xA83D4C in YRpp.
const CURRENT_PLAYER: u32 = 0x00A8_3D4C;
/// `HouseClass::GiveMoney(int)`, 0x4F9950 in YRpp.
const GIVE_MONEY: u32 = 0x004F_9950;

type GiveMoneyFn = unsafe extern "thiscall" fn(house: *mut c_void, amount: i32);

/// Gives `amount` credits to the local player, if a mission is active.
///
/// # Safety
///
/// The engine must have initialized `HouseClass::CurrentPlayer`.
pub(crate) unsafe fn give_money(amount: i32) {
    // SAFETY: 0xA83D4C is the engine's CurrentPlayer pointer. It is null on
    // the main menu and is checked before use.
    let house = unsafe { *(CURRENT_PLAYER as *const *mut c_void) };
    if house.is_null() {
        return;
    }
    // SAFETY: GiveMoney is a thiscall method; `house` is the engine's own
    // HouseClass instance.
    let give_money: GiveMoneyFn = unsafe { core::mem::transmute(GIVE_MONEY as usize) };
    unsafe { give_money(house, amount) };
}
