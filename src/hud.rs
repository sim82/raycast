use crate::prelude::*;

pub fn draw_status_bar<D: Draw + ?Sized>(buffer: &mut D, player: &Player) {
    let mut y = 160;
    draw_string8x8(&format!("health: {}", player.health), buffer, 0, y);
    y += 8;
    draw_string8x8(&format!("ammo: {}", player.weapon.ammo), buffer, 0, y);
    y += 8;
    draw_string8x8(&format!("weapon: {:?}", player.weapon.selected_weapon), buffer, 0, y);
}
