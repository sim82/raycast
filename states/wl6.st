
spawn brown {
    directional 108, stand, ammo
    directional 112, path, ammo
    directional 144, stand, ammo
    directional 148, path, ammo
    directional 180, stand, ammo
    directional 184, path, ammo
}

states brown {
stand:
    state BROWN_S_1, true, 0, ThinkStand, None, stand
path:
    state BROWN_W1_1, true, 20, ThinkPath, None, next
    state BROWN_W1_1, true, 5, None, None, next
    state BROWN_W2_1, true, 15, ThinkPath, None, next
    state BROWN_W3_1, true, 20, ThinkPath, None, next
    state BROWN_W3_1, true, 5, None, None, next
    state BROWN_W4_1, true, 15, ThinkPath, None, path
pain1:
    state BROWN_PAIN_1, false, 10, None, None, chase
pain2:
    state BROWN_PAIN_2, false, 10, None, None, chase
chase:
    state BROWN_W1_1, true, 10, ThinkChase, None, next
    state BROWN_W1_1, true, 3, None, None, next
    state BROWN_W2_1, true, 8, ThinkChase, None, next
    state BROWN_W3_1, true, 10, ThinkChase, None, next
    state BROWN_W3_1, true, 3, None, None, next
    state BROWN_W4_1, true, 8, ThinkChase, None, chase
die:
    state BROWN_DIE_1, false, 15, None, None, next
    state BROWN_DIE_2, false, 15, None, ActionDie, next
    state BROWN_DIE_3, false, 15, None, None, next
dead:
    state BROWN_DEAD, false, 0, None, None, dead
shoot:
    state BROWN_SHOOT1, false, 20, None, None, next
    state BROWN_SHOOT2, false, 20, None, ActionShoot, next
    state BROWN_SHOOT3, false, 20, None, None, chase
}


spawn blue {
    directional 126, stand, ammo
    directional 130, path, ammo
    directional 162, stand, ammo
    directional 166, path, ammo
    directional 198, stand, ammo
    directional 202, path, ammo
}

states blue {
stand:
    state BLUE_S_1, true, 0, ThinkStand, None, stand
path:
    state BLUE_W1_1, true, 20, ThinkPath, None, next
    state BLUE_W1_1, true, 5, None, None, next
    state BLUE_W2_1, true, 15, ThinkPath, None, next
    state BLUE_W3_1, true, 20, ThinkPath, None, next
    state BLUE_W3_1, true, 5, None, None, next
    state BLUE_W4_1, true, 15, ThinkPath, None, path
pain1:
    state BLUE_PAIN_1, false, 10, None, None, chase
pain2:
    state BLUE_PAIN_2, false, 10, None, None, chase
chase:
    state BLUE_W1_1, true, 10, ThinkChase, None, next
    state BLUE_W1_1, true, 3, None, None, next
    state BLUE_W2_1, true, 8, ThinkChase, None, next
    state BLUE_W3_1, true, 10, ThinkChase, None, next
    state BLUE_W3_1, true, 3, None, None, next
    state BLUE_W4_1, true, 8, ThinkChase, None, chase
die:
    state BLUE_DIE_1, false, 15, None, None, next
    state BLUE_DIE_2, false, 15, None, None, next
    state BLUE_DIE_3, false, 15, None, ActionDie, next
dead:
    state BLUE_DEAD, false, 0, None, None, dead

shoot:
    state BLUE_SHOOT1, false, 20, None, None, next
    state BLUE_SHOOT2, false, 10, None, ActionShoot, next
    state BLUE_SHOOT3, false, 10, None, None, next
    state BLUE_SHOOT2, false, 10, None, ActionShoot, next
    state BLUE_SHOOT3, false, 10, None, None, next
    state BLUE_SHOOT2, false, 10, None, ActionShoot, next
    state BLUE_SHOOT3, false, 10, None, None, next
    state BLUE_SHOOT2, false, 10, None, ActionShoot, next
    state BLUE_SHOOT3, false, 10, None, None, chase
}


spawn white {
    directional 116, stand, ammo
    directional 120, path, ammo
    directional 152, stand, ammo
    directional 156, path, ammo
    directional 188, stand, ammo
    directional 192, path, ammo
}

states white {
stand:
    state WHITE_S_1, true, 0, ThinkStand, None, stand
path:
    state WHITE_W1_1, true, 20, ThinkPath, None, next
    state WHITE_W1_1, true, 5, None, None, next
    state WHITE_W2_1, true, 15, ThinkPath, None, next
    state WHITE_W3_1, true, 20, ThinkPath, None, next
    state WHITE_W3_1, true, 5, None, None, next
    state WHITE_W4_1, true, 15, ThinkPath, None, path
pain1:
    state WHITE_PAIN_1, false, 10, None, None, chase
pain2:
    state WHITE_PAIN_2, false, 10, None, None, chase
chase:
    state WHITE_W1_1, true, 10, ThinkChase, None, next
    state WHITE_W1_1, true, 3, None, None, next
    state WHITE_W2_1, true, 8, ThinkChase, None, next
    state WHITE_W3_1, true, 10, ThinkChase, None, next
    state WHITE_W3_1, true, 3, None, None, next
    state WHITE_W4_1, true, 8, ThinkChase, None, chase
die:
    state WHITE_DIE_1, false, 15, None, None, next
    state WHITE_DIE_2, false, 15, None, None, next
    state WHITE_DIE_3, false, 15, None, None, next
    state WHITE_DIE_4, false, 15, None, ActionDie, next
dead:
    state WHITE_DEAD, false, 0, None, None, dead
shoot:
    state WHITE_SHOOT1, false, 20, None, None, next
    state WHITE_SHOOT2, false, 20, None, ActionShoot, next
    state WHITE_SHOOT3, false, 20, None, None, chase
}


spawn rotten {
    directional 216, stand, ammo
    directional 220, path, ammo
    directional 234, stand, ammo
    directional 238, path, ammo
    directional 252, stand, ammo
    directional 256, path, ammo
}


states rotten {
stand:
    state ROTTEN_S_1, true, 0, ThinkStand, None, stand
path:
    state ROTTEN_W1_1, true, 20, ThinkPath, None, next
    state ROTTEN_W1_1, true, 5, None, None, next
    state ROTTEN_W2_1, true, 15, ThinkPath, None, next
    state ROTTEN_W3_1, true, 20, ThinkPath, None, next
    state ROTTEN_W3_1, true, 5, None, None, next
    state ROTTEN_W4_1, true, 15, ThinkPath, None, path
pain1:
    state ROTTEN_PAIN_1, false, 10, None, None, chase
pain2:
    state ROTTEN_PAIN_2, false, 10, None, None, chase
chase:
    state ROTTEN_W1_1, true, 10, ThinkChase, None, next
    state ROTTEN_W1_1, true, 3, None, None, next
    state ROTTEN_W2_1, true, 8, ThinkChase, None, next
    state ROTTEN_W3_1, true, 10, ThinkChase, None, next
    state ROTTEN_W3_1, true, 3, None, None, next
    state ROTTEN_W4_1, true, 8, ThinkChase, None, chase
die:
    state ROTTEN_DIE_1, false, 15, None, None, next
    state ROTTEN_DIE_2, false, 15, None, None, next
    state ROTTEN_DIE_3, false, 15, None, None, next
    state ROTTEN_DIE_4, false, 15, None, ActionDie, next
dead:
    state ROTTEN_DEAD, false, 0, None, None, dead

shoot:
    state ROTTEN_SHOOT1, false, 20, None, ActionShoot, next
    state ROTTEN_SHOOT2, false, 20, None, None, next
    state ROTTEN_SHOOT3, false, 20, None, ActionShoot, next
    state ROTTEN_SHOOT4, false, 20, None, None, chase
}


spawn furry {
    directional 134, stand, none
    directional 138, path, none
    directional 170, stand, none
    directional 174, path, none
    directional 206, stand, none
    directional 210, path, none
}

states furry {
stand:
    state FURRY_W1_1, true, 20, ThinkPath, None, stand
path:
    state FURRY_W1_1, true, 20, ThinkPath, None, next
    state FURRY_W1_1, true, 5, None, None, next
    state FURRY_W2_1, true, 15, ThinkPath, None, next
    state FURRY_W3_1, true, 20, ThinkPath, None, next
    state FURRY_W3_1, true, 5, None, None, next
    state FURRY_W4_1, true, 15, ThinkPath, None, path
pain1:
    state FURRY_DIE_1, false, 10, None, None, chase
pain2:
    state FURRY_DIE_1, false, 10, None, None, chase
chase:
    state FURRY_W1_1, true, 10, ThinkDogChase, None, next
    state FURRY_W1_1, true, 3, None, None, next
    state FURRY_W2_1, true, 8, ThinkDogChase, None, next
    state FURRY_W3_1, true, 10, ThinkDogChase, None, next
    state FURRY_W3_1, true, 3, None, None, next
    state FURRY_W4_1, true, 8, ThinkDogChase, None, chase
die:
    state FURRY_DIE_1, false, 15, None, None, next
    state FURRY_DIE_2, false, 15, None, None, next
    state FURRY_DIE_3, false, 15, None, ActionDie, next
dead:
    state FURRY_DEAD, false, 0, None, None, dead
jump:
    state FURRY_JUMP1, false, 10, None, None, next
    state FURRY_JUMP2, false, 10, None, ActionBite, next
    state FURRY_JUMP3, false, 10, None, None, next
    state FURRY_JUMP1, false, 10, None, None, next
    state FURRY_W1_1, false, 10, None, None, chase
}


spawn hans {
    undirectional 214, stand, silver_key
}

states hans {
stand:
    state BOSS_W1, false, 0, ThinkStand, None, stand
path:
    state BOSS_W1, false, 20, ThinkPath, None, next
    state BOSS_W1, false, 5, None, None, next
    state BOSS_W2, false, 15, ThinkPath, None, next
    state BOSS_W3, false, 20, ThinkPath, None, next
    state BOSS_W3, false, 5, None, None, next
    state BOSS_W4, false, 15, ThinkPath, None, path
pain1:
    state BOSS_DIE1, false, 10, None, None, chase
pain2:
    state BOSS_DIE1, false, 10, None, None, chase
chase:
    state BOSS_W1, false, 10, ThinkChase, None, next
    state BOSS_W1, false, 3, None, None, next
    state BOSS_W2, false, 8, ThinkChase, None, next
    state BOSS_W3, false, 10, ThinkChase, None, next
    state BOSS_W3, false, 3, None, None, next
    state BOSS_W4, false, 8, ThinkChase, None, chase
die:
    state BOSS_DIE1, false, 15, None, None, next
    state BOSS_DIE2, false, 15, None, ActionDie, next
    state BOSS_DIE3, false, 15, None, None, dead
dead:
    state BOSS_DEAD, false, 0, None, None, dead
shoot:
    state BOSS_SHOOT1, false, 20, None, None, next
    state BOSS_SHOOT2, false, 20, None, ActionShoot, next
    state BOSS_SHOOT3, false, 20, None, None, chase
}

spawn good_dr {
    undirectional 196, stand, None
}

states good_dr {
stand:
    state GOOD_DR_W1, false, 0, ThinkStand, None, stand
path:
    state GOOD_DR_W1, false, 20, ThinkPath, None, next
    state GOOD_DR_W1, false, 5, None, None, next
    state GOOD_DR_W2, false, 15, ThinkPath, None, next
    state GOOD_DR_W3, false, 20, ThinkPath, None, next
    state GOOD_DR_W3, false, 5, None, None, next
    state GOOD_DR_W4, false, 15, ThinkPath, None, path
pain1:
    state GOOD_DR_W1, false, 10, None, None, chase
pain2:
    state GOOD_DR_W1, false, 10, None, None, chase
chase:
    state GOOD_DR_W1, false, 10, ThinkChase, None, next
    state GOOD_DR_W1, false, 3, None, None, next
    state GOOD_DR_W2, false, 8, ThinkChase, None, next
    state GOOD_DR_W3, false, 10, ThinkChase, None, next
    state GOOD_DR_W3, false, 3, None, None, next
    state GOOD_DR_W4, false, 8, ThinkChase, None, chase
die:
    state GOOD_DR_DIE1, false, 15, None, None, next
    state GOOD_DR_DIE2, false, 15, None, ActionDie, next
    state GOOD_DR_DIE3, false, 15, None, None, dead
dead:
    state GOOD_DR_DEAD, false, 0, None, None, dead
shoot:
    state GOOD_DR_SHOOT1, false, 20, None, ActionShoot, next
    state GOOD_DR_SHOOT2, false, 20, None, None, chase
}


spawn grofaz {
    undirectional 224, chase, None
}

states grofaz {
stand:
    state GROFAZ_W1, false, 0, ThinkStand, None, stand
path:
    state GROFAZ_W1, false, 20, ThinkPath, None, next
    state GROFAZ_W1, false, 5, None, None, next
    state GROFAZ_W2, false, 15, ThinkPath, None, next
    state GROFAZ_W3, false, 20, ThinkPath, None, next
    state GROFAZ_W3, false, 5, None, None, next
    state GROFAZ_W4, false, 15, ThinkPath, None, path
pain1:
    state GROFAZ_W1, false, 10, None, None, chase
pain2:
    state GROFAZ_W1, false, 10, None, None, chase
chase:
    state GROFAZ_W1, false, 10, ThinkChase, None, next
    state GROFAZ_W1, false, 3, None, None, next
    state GROFAZ_W2, false, 8, ThinkChase, None, next
    state GROFAZ_W3, false, 10, ThinkChase, None, next
    state GROFAZ_W3, false, 3, None, None, next
    state GROFAZ_W4, false, 8, ThinkChase, None, chase
die:
    state GROFAZ_DIE1, false, 15, None, None, next
    state GROFAZ_DIE2, false, 15, None, None, next
    state GROFAZ_DIE3, false, 15, None, None, next
    state GROFAZ_DIE4, false, 15, None, None, next
    state GROFAZ_DIE5, false, 15, None, None, next
    state GROFAZ_DIE6, false, 15, None, ActionDie, next
    state GROFAZ_DIE7, false, 15, None, None, next
dead:
    state GROFAZ_DEAD, false, 0, None, None, dead
shoot:
    state GROFAZ_SHOOT1, false, 20, None, None, next
    state GROFAZ_SHOOT2, false, 20, None, ActionShoot, next
    state GROFAZ_SHOOT3, false, 20, None, None, chase
}


spawn mecha {
    undirectional 178, stand, grofaz
}

states mecha {
stand:
    state MECHA_W1, false, 0, ThinkStand, None, stand
path:
    state MECHA_W1, false, 20, ThinkPath, None, next
    state MECHA_W1, false, 5, None, None, next
    state MECHA_W2, false, 15, ThinkPath, None, next
    state MECHA_W3, false, 20, ThinkPath, None, next
    state MECHA_W3, false, 5, None, None, next
    state MECHA_W4, false, 15, ThinkPath, None, path
pain1:
    state MECHA_DIE1, false, 10, None, None, chase
pain2:
    state MECHA_DIE1, false, 10, None, None, chase
chase:
    state MECHA_W1, false, 10, ThinkChase, None, next
    state MECHA_W1, false, 3, None, None, next
    state MECHA_W2, false, 8, ThinkChase, None, next
    state MECHA_W3, false, 10, ThinkChase, None, next
    state MECHA_W3, false, 3, None, None, next
    state MECHA_W4, false, 8, ThinkChase, None, chase
die:
    state MECHA_DIE1, false, 15, None, None, next
    state MECHA_DIE2, false, 15, None, None, next
    state MECHA_DIE3, false, 15, None, ActionDie, next
    state MECHA_DIE3, false, 0, None, None, next
dead:
    state MECHA_DEAD, false, 0, None, None, dead
shoot:
    state MECHA_SHOOT1, false, 20, None, None, next
    state MECHA_SHOOT2, false, 20, None, ActionShoot, next
    state MECHA_SHOOT3, false, 20, None, None, chase
} 

spawn alp {
    undirectional 160, stand, None
}

states alp {
stand:
    state ALP_W1, false, 0, ThinkStand, None, stand
path:
    state ALP_W1, false, 20, ThinkPath, None, next
    state ALP_W1, false, 5, None, None, next
    state ALP_W2, false, 15, ThinkPath, None, next
    state ALP_W3, false, 20, ThinkPath, None, next
    state ALP_W3, false, 5, None, None, next
    state ALP_W4, false, 15, ThinkPath, None, path
pain1:
    state ALP_DIE1, false, 10, None, None, chase
pain2:
    state ALP_DIE1, false, 10, None, None, chase
chase:
    state ALP_W1, false, 10, ThinkChase, None, next
    state ALP_W1, false, 3, None, None, next
    state ALP_W2, false, 8, ThinkChase, None, next
    state ALP_W3, false, 10, ThinkChase, None, next
    state ALP_W3, false, 3, None, None, next
    state ALP_W4, false, 8, ThinkChase, None, chase
die:
    state ALP_DIE1, false, 15, None, None, next
    state ALP_DIE2, false, 15, None, None, next
    state ALP_DIE3, false, 15, None, None, next
    state ALP_DIE4, false, 15, None, ActionDie, next
    state ALP_DIE5, false, 15, None, None, next
dead:
    state ALP_DEAD, false, 0, None, None, dead
shoot:
    state ALP_W1, false, 20, None, ActionShoot, next
    state ALP_W1, false, 20, None, None, chase
}


spawn hermann {
    undirectional 215, stand, None
}

states hermann {
stand:
    state HERMANN_W1, false, 0, ThinkStand, None, stand
path:
    state HERMANN_W1, false, 20, ThinkPath, None, next
    state HERMANN_W1, false, 5, None, None, next
    state HERMANN_W2, false, 15, ThinkPath, None, next
    state HERMANN_W3, false, 20, ThinkPath, None, next
    state HERMANN_W3, false, 5, None, None, next
    state HERMANN_W4, false, 15, ThinkPath, None, path
pain1:
    state HERMANN_W1, false, 10, None, None, chase
pain2:
    state HERMANN_W1, false, 10, None, None, chase
chase:
    state HERMANN_W1, false, 10, ThinkChase, None, next
    state HERMANN_W1, false, 3, None, None, next
    state HERMANN_W2, false, 8, ThinkChase, None, next
    state HERMANN_W3, false, 10, ThinkChase, None, next
    state HERMANN_W3, false, 3, None, None, next
    state HERMANN_W4, false, 8, ThinkChase, None, chase
die:
    state HERMANN_DIE1, false, 15, None, None, next
    state HERMANN_DIE2, false, 15, None, ActionDie, next
    state HERMANN_DIE3, false, 15, None, None, dead
dead:
    state HERMANN_DEAD, false, 0, None, None, dead
shoot:
    state HERMANN_SHOOT1, false, 20, None, ActionShoot, next
    state HERMANN_SHOOT2, false, 20, None, None, chase
}

spawn gretel {
    undirectional 197, stand, None
}

states gretel {
stand:
    state GRETEL_W1, false, 0, ThinkStand, None, stand
path:
    state GRETEL_W1, false, 20, ThinkPath, None, next
    state GRETEL_W1, false, 5, None, None, next
    state GRETEL_W2, false, 15, ThinkPath, None, next
    state GRETEL_W3, false, 20, ThinkPath, None, next
    state GRETEL_W3, false, 5, None, None, next
    state GRETEL_W4, false, 15, ThinkPath, None, path
pain1:
    state GRETEL_W1, false, 10, None, None, chase
pain2:
    state GRETEL_W1, false, 10, None, None, chase
chase:
    state GRETEL_W1, false, 10, ThinkChase, None, next
    state GRETEL_W1, false, 3, None, None, next
    state GRETEL_W2, false, 8, ThinkChase, None, next
    state GRETEL_W3, false, 10, ThinkChase, None, next
    state GRETEL_W3, false, 3, None, None, next
    state GRETEL_W4, false, 8, ThinkChase, None, chase
die:
    state GRETEL_DIE1, false, 15, None, None, next
    state GRETEL_DIE2, false, 15, None, ActionDie, next
    state GRETEL_DIE3, false, 15, None, None, dead
dead:
    state GRETEL_DEAD, false, 0, None, None, dead
shoot:
    state GRETEL_SHOOT1, false, 20, None, ActionShoot, next
    state GRETEL_SHOOT2, false, 20, None, None, chase
}

spawn fettgeischt {
    undirectional 179, stand, None
}

states fettgeischt {
stand:
    state FETTGEISCHT_W1, false, 0, ThinkStand, None, stand
path:
    state FETTGEISCHT_W1, false, 20, ThinkPath, None, next
    state FETTGEISCHT_W1, false, 5, None, None, next
    state FETTGEISCHT_W2, false, 15, ThinkPath, None, next
    state FETTGEISCHT_W3, false, 20, ThinkPath, None, next
    state FETTGEISCHT_W3, false, 5, None, None, next
    state FETTGEISCHT_W4, false, 15, ThinkPath, None, path
pain1:
    state FETTGEISCHT_W1, false, 10, None, None, chase
pain2:
    state FETTGEISCHT_W1, false, 10, None, None, chase
chase:
    state FETTGEISCHT_W1, false, 10, ThinkChase, None, next
    state FETTGEISCHT_W1, false, 3, None, None, next
    state FETTGEISCHT_W2, false, 8, ThinkChase, None, next
    state FETTGEISCHT_W3, false, 10, ThinkChase, None, next
    state FETTGEISCHT_W3, false, 3, None, None, next
    state FETTGEISCHT_W4, false, 8, ThinkChase, None, chase
die:
    state FETTGEISCHT_DIE1, false, 15, None, None, next
    state FETTGEISCHT_DIE2, false, 15, None, ActionDie, next
    state FETTGEISCHT_DIE3, false, 15, None, None, dead
dead:
    state FETTGEISCHT_DEAD, false, 0, None, None, dead
shoot:
    state FETTGEISCHT_SHOOT1, false, 20, None, None, next
    state FETTGEISCHT_SHOOT2, false, 20, None, None, next
    state FETTGEISCHT_SHOOT3, false, 20, None, ActionShoot, next
    state FETTGEISCHT_SHOOT4, false, 20, None, None, chase
}



states weapon_knife {
lower:
ready:
    state SPR_KNIFEREADY, false, 0, ThinkStand, None, ready
attack:
    state SPR_KNIFEATK1, false, 3, None, ActionDie, next
    state SPR_KNIFEATK2, false, 3, None, None, next
    state SPR_KNIFEATK3, false, 10, None, None, next
    state SPR_KNIFEATK4, false, 10, None, None, ready
}

states weapon_gun {
lower:
    state SPR_PISTOLATK4, false, 10, None, None, next
ready:
    state SPR_PISTOLREADY, false, 0, ThinkStand, None, ready
attack:
    state SPR_PISTOLATK1, false, 1, ThinkPath, None, next
repeat:
    state SPR_PISTOLATK2, false, 3, None, ActionShoot, next
    state SPR_PISTOLATK3, false, 10, None, None, lower
}

states weapon_machinegun {
lower:
    state SPR_MACHINEGUNATK4, false, 3, None, None, ready
ready:
    state SPR_MACHINEGUNREADY, false, 0, ThinkStand, None, ready
attack:
    state SPR_MACHINEGUNATK1, false, 4, ThinkPath, None, next
repeat:
    state SPR_MACHINEGUNATK2, false, 4, None, ActionShoot, next
    state SPR_MACHINEGUNATK3, false, 4, ThinkPath, None, repeat
}


states weapon_chaingun {
lower:
    state SPR_CHAINATK4, false, 4, None, None, ready
ready:
    state SPR_CHAINREADY, false, 0, ThinkStand, None, ready
attack:
    state SPR_CHAINATK1, false, 4, ThinkPath, None, next
repeat:
    state SPR_CHAINATK2, false, 4, None, ActionShoot, next
    state SPR_CHAINATK3, false, 4, ThinkPath, ActionShoot, repeat
}

states door {
closed:
    state DEMO, false, 0, ThinkStand, None, closed
opened: 
    state DEMO, false, 300, ThinkPath, None, blocked
close:
    state DEMO, false, 64, DecOpen, None, closed
open:
    state DEMO, false, 64, IncOpen, None, opened
blocked: 
    state DEMO, false, 0, ActionShoot, None, blocked
}

function IncOpen {
    loadi32 0
    loadii32 1024
    add
    storei32 0
}
function DecOpen {
    loadi32 0
    loadii32 -1024
    add
    storei32 0
}
function ThinkStand {
    loadiu8 THINK_STAND
    call
}
function ThinkPath {
    loadiu8 THINK_PATH
    call
}
function ThinkChase {
    loadiu8 THINK_CHASE
    call
}
function ThinkDogChase {
    loadiu8 THINK_DOG_CHASE
    call
}
function ActionDie {
    loadiu8 ACTION_DIE
    call
}
function ActionShoot {
    loadiu8 ACTION_SHOOT
    call
}
function ActionBite {
    loadiu8 ACTION_BITE
    call
}
function None {
}

enum
{
    FUNC_NONE,
    THINK_STAND,
    THINK_PATH,
    THINK_CHASE,
    THINK_DOG_CHASE,
    ACTION_DIE,
    ACTION_SHOOT,
    ACTION_BITE,
}
enum
{
    SPR_NONE,
    DEMO,
    DEATHCAM,
    STAT_0,STAT_1,STAT_2,STAT_3,
    STAT_4,STAT_5,STAT_6,STAT_7,

    STAT_8,STAT_9,STAT_10,STAT_11,
    STAT_12,STAT_13,STAT_14,STAT_15,

    STAT_16,STAT_17,STAT_18,STAT_19,
    STAT_20,STAT_21,STAT_22,STAT_23,

    STAT_24,STAT_25,STAT_26,STAT_27,
    STAT_28,STAT_29,STAT_30,STAT_31,

    STAT_32,STAT_33,STAT_34,STAT_35,
    STAT_36,STAT_37,STAT_38,STAT_39,

    STAT_40,STAT_41,STAT_42,STAT_43,
    STAT_44,STAT_45,STAT_46,STAT_47,


    BROWN_S_1,BROWN_S_2,BROWN_S_3,BROWN_S_4,
    BROWN_S_5,BROWN_S_6,BROWN_S_7,BROWN_S_8,

    BROWN_W1_1,BROWN_W1_2,BROWN_W1_3,BROWN_W1_4,
    BROWN_W1_5,BROWN_W1_6,BROWN_W1_7,BROWN_W1_8,

    BROWN_W2_1,BROWN_W2_2,BROWN_W2_3,BROWN_W2_4,
    BROWN_W2_5,BROWN_W2_6,BROWN_W2_7,BROWN_W2_8,

    BROWN_W3_1,BROWN_W3_2,BROWN_W3_3,BROWN_W3_4,
    BROWN_W3_5,BROWN_W3_6,BROWN_W3_7,BROWN_W3_8,

    BROWN_W4_1,BROWN_W4_2,BROWN_W4_3,BROWN_W4_4,
    BROWN_W4_5,BROWN_W4_6,BROWN_W4_7,BROWN_W4_8,

    BROWN_PAIN_1,BROWN_DIE_1,BROWN_DIE_2,BROWN_DIE_3,
    BROWN_PAIN_2,BROWN_DEAD,

    BROWN_SHOOT1,BROWN_SHOOT2,BROWN_SHOOT3,

    FURRY_W1_1,FURRY_W1_2,FURRY_W1_3,FURRY_W1_4,
    FURRY_W1_5,FURRY_W1_6,FURRY_W1_7,FURRY_W1_8,

    FURRY_W2_1,FURRY_W2_2,FURRY_W2_3,FURRY_W2_4,
    FURRY_W2_5,FURRY_W2_6,FURRY_W2_7,FURRY_W2_8,

    FURRY_W3_1,FURRY_W3_2,FURRY_W3_3,FURRY_W3_4,
    FURRY_W3_5,FURRY_W3_6,FURRY_W3_7,FURRY_W3_8,

    FURRY_W4_1,FURRY_W4_2,FURRY_W4_3,FURRY_W4_4,
    FURRY_W4_5,FURRY_W4_6,FURRY_W4_7,FURRY_W4_8,

    FURRY_DIE_1,FURRY_DIE_2,FURRY_DIE_3,FURRY_DEAD,
    FURRY_JUMP1,FURRY_JUMP2,FURRY_JUMP3,

    BLUE_S_1,BLUE_S_2,BLUE_S_3,BLUE_S_4,
    BLUE_S_5,BLUE_S_6,BLUE_S_7,BLUE_S_8,

    BLUE_W1_1,BLUE_W1_2,BLUE_W1_3,BLUE_W1_4,
    BLUE_W1_5,BLUE_W1_6,BLUE_W1_7,BLUE_W1_8,

    BLUE_W2_1,BLUE_W2_2,BLUE_W2_3,BLUE_W2_4,
    BLUE_W2_5,BLUE_W2_6,BLUE_W2_7,BLUE_W2_8,

    BLUE_W3_1,BLUE_W3_2,BLUE_W3_3,BLUE_W3_4,
    BLUE_W3_5,BLUE_W3_6,BLUE_W3_7,BLUE_W3_8,

    BLUE_W4_1,BLUE_W4_2,BLUE_W4_3,BLUE_W4_4,
    BLUE_W4_5,BLUE_W4_6,BLUE_W4_7,BLUE_W4_8,

    BLUE_PAIN_1,BLUE_DIE_1,BLUE_DIE_2,BLUE_DIE_3,
    BLUE_PAIN_2,BLUE_DEAD,

    BLUE_SHOOT1,BLUE_SHOOT2,BLUE_SHOOT3,

    ROTTEN_S_1,ROTTEN_S_2,ROTTEN_S_3,ROTTEN_S_4,
    ROTTEN_S_5,ROTTEN_S_6,ROTTEN_S_7,ROTTEN_S_8,

    ROTTEN_W1_1,ROTTEN_W1_2,ROTTEN_W1_3,ROTTEN_W1_4,
    ROTTEN_W1_5,ROTTEN_W1_6,ROTTEN_W1_7,ROTTEN_W1_8,

    ROTTEN_W2_1,ROTTEN_W2_2,ROTTEN_W2_3,ROTTEN_W2_4,
    ROTTEN_W2_5,ROTTEN_W2_6,ROTTEN_W2_7,ROTTEN_W2_8,

    ROTTEN_W3_1,ROTTEN_W3_2,ROTTEN_W3_3,ROTTEN_W3_4,
    ROTTEN_W3_5,ROTTEN_W3_6,ROTTEN_W3_7,ROTTEN_W3_8,

    ROTTEN_W4_1,ROTTEN_W4_2,ROTTEN_W4_3,ROTTEN_W4_4,
    ROTTEN_W4_5,ROTTEN_W4_6,ROTTEN_W4_7,ROTTEN_W4_8,

    ROTTEN_PAIN_1,ROTTEN_DIE_1,ROTTEN_DIE_2,ROTTEN_DIE_3,
    ROTTEN_PAIN_2,ROTTEN_DIE_4,ROTTEN_DEAD,

    ROTTEN_SHOOT1,ROTTEN_SHOOT2,ROTTEN_SHOOT3,ROTTEN_SHOOT4,

    WHITE_S_1,WHITE_S_2,WHITE_S_3,WHITE_S_4,
    WHITE_S_5,WHITE_S_6,WHITE_S_7,WHITE_S_8,

    WHITE_W1_1,WHITE_W1_2,WHITE_W1_3,WHITE_W1_4,
    WHITE_W1_5,WHITE_W1_6,WHITE_W1_7,WHITE_W1_8,

    WHITE_W2_1,WHITE_W2_2,WHITE_W2_3,WHITE_W2_4,
    WHITE_W2_5,WHITE_W2_6,WHITE_W2_7,WHITE_W2_8,

    WHITE_W3_1,WHITE_W3_2,WHITE_W3_3,WHITE_W3_4,
    WHITE_W3_5,WHITE_W3_6,WHITE_W3_7,WHITE_W3_8,

    WHITE_W4_1,WHITE_W4_2,WHITE_W4_3,WHITE_W4_4,
    WHITE_W4_5,WHITE_W4_6,WHITE_W4_7,WHITE_W4_8,

    WHITE_PAIN_1,WHITE_DIE_1,WHITE_DIE_2,WHITE_DIE_3,
    WHITE_PAIN_2,WHITE_DIE_4,WHITE_DEAD,

    WHITE_SHOOT1,WHITE_SHOOT2,WHITE_SHOOT3,

    BLINKY_W1,BLINKY_W2,PINKY_W1,PINKY_W2,
    CLYDE_W1,CLYDE_W2,INKY_W1,INKY_W2,
    
    BOSS_W1,BOSS_W2,BOSS_W3,BOSS_W4,
    BOSS_SHOOT1,BOSS_SHOOT2,BOSS_SHOOT3,BOSS_DEAD,
    BOSS_DIE1,BOSS_DIE2,BOSS_DIE3,
    
    GOOD_DR_W1,GOOD_DR_W2,GOOD_DR_W3,GOOD_DR_W4,
    GOOD_DR_SHOOT1,GOOD_DR_SHOOT2,
    GOOD_DR_DIE1,GOOD_DR_DIE2,GOOD_DR_DIE3,GOOD_DR_DEAD,
    PARTY_O1,PARTY_O2,PARTY_O3,PARTY_O4,
    
    ALP_W1,ALP_W2,ALP_W3,ALP_W4,
    ALP_SHOOT,ALP_FIRE1,ALP_FIRE2,
    ALP_DIE1,ALP_DIE2,ALP_DIE3,ALP_DIE4,
    ALP_DIE5,ALP_DEAD,
    
    MECHA_W1,MECHA_W2,MECHA_W3,MECHA_W4,
    MECHA_SHOOT1,MECHA_SHOOT2,MECHA_SHOOT3,MECHA_DEAD,
    MECHA_DIE1,MECHA_DIE2,MECHA_DIE3,
    
    GROFAZ_W1,GROFAZ_W2,GROFAZ_W3,GROFAZ_W4,
    GROFAZ_SHOOT1,GROFAZ_SHOOT2,GROFAZ_SHOOT3,GROFAZ_DEAD,
    GROFAZ_DIE1,GROFAZ_DIE2,GROFAZ_DIE3,GROFAZ_DIE4,
    GROFAZ_DIE5,GROFAZ_DIE6,GROFAZ_DIE7,
    
    HERMANN_W1,HERMANN_W2,HERMANN_W3,HERMANN_W4,
    HERMANN_SHOOT1,HERMANN_SHOOT2,
    HERMANN_DIE1,HERMANN_DIE2,HERMANN_DIE3,HERMANN_DEAD,
    
    ROCKET_1,ROCKET_2,ROCKET_3,ROCKET_4,
    ROCKET_5,ROCKET_6,ROCKET_7,ROCKET_8,
    
    SMOKE_1,SMOKE_2,SMOKE_3,SMOKE_4,
    BOOM_1,BOOM_2,BOOM_3,

    GRETEL_W1,GRETEL_W2,GRETEL_W3,GRETEL_W4,
    GRETEL_SHOOT1,GRETEL_SHOOT2,GRETEL_SHOOT3,GRETEL_DEAD,
    GRETEL_DIE1,GRETEL_DIE2,GRETEL_DIE3,
   
    FETTGEISCHT_W1,FETTGEISCHT_W2,FETTGEISCHT_W3,FETTGEISCHT_W4,
    FETTGEISCHT_SHOOT1,FETTGEISCHT_SHOOT2,FETTGEISCHT_SHOOT3,FETTGEISCHT_SHOOT4,
    FETTGEISCHT_DIE1,FETTGEISCHT_DIE2,FETTGEISCHT_DIE3,FETTGEISCHT_DEAD,
   
    SELF1_W1,SELF_W2,SELF_W3,SELF_W4,
    SELF_JUMP1,SELF_JUMP2,SELF_JUMP3,SELF_JUMP4,
   
    SPR_KNIFEREADY,SPR_KNIFEATK1,SPR_KNIFEATK2,SPR_KNIFEATK3,
    SPR_KNIFEATK4,
   
    SPR_PISTOLREADY,SPR_PISTOLATK1,SPR_PISTOLATK2,SPR_PISTOLATK3,
    SPR_PISTOLATK4,
   
    SPR_MACHINEGUNREADY,SPR_MACHINEGUNATK1,SPR_MACHINEGUNATK2,SPR_MACHINEGUNATK3,
    SPR_MACHINEGUNATK4,
   
    SPR_CHAINREADY,SPR_CHAINATK1,SPR_CHAINATK2,SPR_CHAINATK3,
    SPR_CHAINATK4,

}