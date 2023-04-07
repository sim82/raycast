states brown {
stand:
    state BROWN_S_1, true, 0, Stand, None, stand
path:
    state BROWN_W1_1, true, 20, Path, None, next
    state BROWN_W1_1, true, 5, None, None, next
    state BROWN_W2_1, true, 15, Path, None, next
    state BROWN_W3_1, true, 20, Path, None, next
    state BROWN_W3_1, true, 5, None, None, next
    state BROWN_W4_1, true, 15, Path, None, path
pain1:
    state BROWN_PAIN_1, false, 10, None, None, chase
pain2:
    state BROWN_PAIN_2, false, 10, None, None, chase
chase:
    state BROWN_W1_1, true, 10, Chase, None, next
    state BROWN_W1_1, true, 3, None, None, next
    state BROWN_W2_1, true, 8, Chase, None, next
    state BROWN_W3_1, true, 10, Chase, None, next
    state BROWN_W3_1, true, 3, None, None, next
    state BROWN_W4_1, true, 8, Chase, None, chase
die:
    state BROWN_DIE_1, false, 15, None, None, next
    state BROWN_DIE_2, false, 15, None, None, next
    state BROWN_DIE_3, false, 15, None, Die, next
dead:
    state BROWN_DEAD, false, 0, None, None, dead
shoot:
    state BROWN_SHOOT1, false, 20, None, None, next
    state BROWN_SHOOT2, false, 20, None, Shoot, next
    state BROWN_SHOOT3, false, 20, None, None, chase
}

states blue {
stand:
    state BLUE_S_1, true, 0, Stand, None, stand
path:
    state BLUE_W1_1, true, 20, Path, None, next
    state BLUE_W1_1, true, 5, None, None, next
    state BLUE_W2_1, true, 15, Path, None, next
    state BLUE_W3_1, true, 20, Path, None, next
    state BLUE_W3_1, true, 5, None, None, next
    state BLUE_W4_1, true, 15, Path, None, path
pain1:
    state BLUE_PAIN_1, false, 10, None, None, chase
pain2:
    state BLUE_PAIN_2, false, 10, None, None, chase
chase:
    state BLUE_W1_1, true, 10, Chase, None, next
    state BLUE_W1_1, true, 3, None, None, next
    state BLUE_W2_1, true, 8, Chase, None, next
    state BLUE_W3_1, true, 10, Chase, None, next
    state BLUE_W3_1, true, 3, None, None, next
    state BLUE_W4_1, true, 8, Chase, None, chase
die:
    state BLUE_DIE_1, false, 15, None, None, next
    state BLUE_DIE_2, false, 15, None, None, next
    state BLUE_DIE_3, false, 15, None, None, next
dead:
    state BLUE_DEAD, false, 0, None, Die, dead

shoot:
    state BLUE_SHOOT1, false, 20, None, None, next
    state BLUE_SHOOT2, false, 10, None, Shoot, next
    state BLUE_SHOOT3, false, 10, None, None, next
    state BLUE_SHOOT2, false, 10, None, Shoot, next
    state BLUE_SHOOT3, false, 10, None, None, next
    state BLUE_SHOOT2, false, 10, None, Shoot, next
    state BLUE_SHOOT3, false, 10, None, None, next
    state BLUE_SHOOT2, false, 10, None, Shoot, next
    state BLUE_SHOOT3, false, 10, None, None, chase
}

states white {
stand:
    state WHITE_S_1, true, 0, Stand, None, stand
path:
    state WHITE_W1_1, true, 20, Path, None, next
    state WHITE_W1_1, true, 5, None, None, next
    state WHITE_W2_1, true, 15, Path, None, next
    state WHITE_W3_1, true, 20, Path, None, next
    state WHITE_W3_1, true, 5, None, None, next
    state WHITE_W4_1, true, 15, Path, None, path
pain1:
    state WHITE_PAIN_1, false, 10, None, None, chase
pain2:
    state WHITE_PAIN_2, false, 10, None, None, chase
chase:
    state WHITE_W1_1, true, 10, Chase, None, next
    state WHITE_W1_1, true, 3, None, None, next
    state WHITE_W2_1, true, 8, Chase, None, next
    state WHITE_W3_1, true, 10, Chase, None, next
    state WHITE_W3_1, true, 3, None, None, next
    state WHITE_W4_1, true, 8, Chase, None, chase
die:
    state WHITE_DIE_1, false, 15, None, None, next
    state WHITE_DIE_2, false, 15, None, None, next
    state WHITE_DIE_3, false, 15, None, None, next
dead:
    state WHITE_DEAD, false, 0, None, Die, dead
shoot:
    state WHITE_SHOOT1, false, 20, None, None, next
    state WHITE_SHOOT2, false, 20, None, Shoot, next
    state WHITE_SHOOT3, false, 20, None, None, chase
}

states rotten {
stand:
    state ROTTEN_S_1, true, 0, Stand, None, stand
path:
    state ROTTEN_W1_1, true, 20, Path, None, next
    state ROTTEN_W1_1, true, 5, None, None, next
    state ROTTEN_W2_1, true, 15, Path, None, next
    state ROTTEN_W3_1, true, 20, Path, None, next
    state ROTTEN_W3_1, true, 5, None, None, next
    state ROTTEN_W4_1, true, 15, Path, None, path
pain1:
    state ROTTEN_PAIN_1, false, 10, None, None, chase
pain2:
    state ROTTEN_PAIN_2, false, 10, None, None, chase
chase:
    state ROTTEN_W1_1, true, 10, Chase, None, next
    state ROTTEN_W1_1, true, 3, None, None, next
    state ROTTEN_W2_1, true, 8, Chase, None, next
    state ROTTEN_W3_1, true, 10, Chase, None, next
    state ROTTEN_W3_1, true, 3, None, None, next
    state ROTTEN_W4_1, true, 8, Chase, None, chase
die:
    state ROTTEN_DIE_1, false, 15, None, None, next
    state ROTTEN_DIE_2, false, 15, None, None, next
    state ROTTEN_DIE_3, false, 15, None, None, next
dead:
    state ROTTEN_DEAD, false, 0, None, Die, dead

shoot:
    state ROTTEN_SHOOT1, false, 20, None, None, next
    state ROTTEN_SHOOT2, false, 20, None, Shoot, next
    state ROTTEN_SHOOT3, false, 20, None, None, chase
}

states furry {
stand:
    state FURRY_W1_1, true, 20, Path, None, stand
path:
    state FURRY_W1_1, true, 20, Path, None, next
    state FURRY_W1_1, true, 5, None, None, next
    state FURRY_W2_1, true, 15, Path, None, next
    state FURRY_W3_1, true, 20, Path, None, next
    state FURRY_W3_1, true, 5, None, None, next
    state FURRY_W4_1, true, 15, Path, None, path
pain1:
    state FURRY_DIE_1, false, 10, None, None, chase
pain2:
    state FURRY_DIE_1, false, 10, None, None, chase
chase:
    state FURRY_W1_1, true, 10, DogChase, None, next
    state FURRY_W1_1, true, 3, None, None, next
    state FURRY_W2_1, true, 8, DogChase, None, next
    state FURRY_W3_1, true, 10, DogChase, None, next
    state FURRY_W3_1, true, 3, None, None, next
    state FURRY_W4_1, true, 8, DogChase, None, chase
die:
    state FURRY_DIE_1, false, 15, None, None, next
    state FURRY_DIE_2, false, 15, None, None, next
    state FURRY_DIE_3, false, 15, None, None, next
dead:
    state FURRY_DEAD, false, 0, None, Die, dead
jump:
    state FURRY_JUMP1, false, 10, None, None, next
    state FURRY_JUMP2, false, 10, None, Bite, next
    state FURRY_JUMP3, false, 10, None, None, next
    state FURRY_JUMP1, false, 10, None, None, next
    state FURRY_W1_1, false, 10, None, None, chase
}

states weapon_knife {
lower:
ready:
    state SPR_KNIFEREADY, false, 0, Stand, None, ready
attack:
    state SPR_KNIFEATK1, false, 3, None, Shoot, next
    state SPR_KNIFEATK2, false, 3, None, None, next
    state SPR_KNIFEATK3, false, 10, None, None, next
    state SPR_KNIFEATK4, false, 10, None, None, ready
}

states weapon_gun {
lower:
    state SPR_PISTOLATK4, false, 10, None, None, next
ready:
    state SPR_PISTOLREADY, false, 0, Stand, None, ready
attack:
    state SPR_PISTOLATK1, false, 2, Path, None, next
repeat:
    state SPR_PISTOLATK2, false, 3, Path, Shoot, next
    state SPR_PISTOLATK3, false, 10, None, None, lower
}

states weapon_machinegun {
lower:
    state SPR_MACHINEGUNATK4, false, 3, None, None, ready
ready:
    state SPR_MACHINEGUNREADY, false, 0, Stand, None, ready
attack:
    state SPR_MACHINEGUNATK1, false, 4, Path, None, next
repeat:
    state SPR_MACHINEGUNATK2, false, 4, Path, Shoot, next
    state SPR_MACHINEGUNATK3, false, 4, Path, None, repeat
}


states weapon_chaingun {
lower:
    state SPR_CHAINATK4, false, 4, None, None, ready
ready:
    state SPR_CHAINREADY, false, 0, Stand, None, ready
attack:
    state SPR_CHAINATK4, false, 4, Path, None, next
    state SPR_CHAINATK1, false, 4, Path, None, next
repeat:
    state SPR_CHAINATK2, false, 4, Path, Shoot, next
    state SPR_CHAINATK3, false, 4, Path, Shoot, repeat
}

states door {
closed:
    state DEMO, false, 0, Stand, None, closed
opened: 
    state DEMO, false, 300, Path, None, blocked
close:
    state DEMO, false, 64, Chase, None, closed
open:
    state DEMO, false, 64, DogChase, None, opened
blocked: 
    state DEMO, false, 0, Shoot, None, blocked
}

enum
{
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

    SPR_BLINKY_W1,SPR_BLINKY_W2,SPR_PINKY_W1,SPR_PINKY_W2,
    SPR_CLYDE_W1,SPR_CLYDE_W2,SPR_INKY_W1,SPR_INKY_W2,
    
    SPR_BOSS_W1,SPR_BOSS_W2,SPR_BOSS_W3,SPR_BOSS_W4,
    SPR_BOSS_SHOOT1,SPR_BOSS_SHOOT2,SPR_BOSS_SHOOT3,SPR_BOSS_DEAD,
    SPR_BOSS_DIE1,SPR_BOSS_DIE2,SPR_BOSS_DIE3,
    
    SPR_GOOD_DR_W1,SPR_GOOD_DR_W2,SPR_GOOD_DR_W3,SPR_GOOD_DR_W4,
    SPR_GOOD_DR_SHOOT1,SPR_GOOD_DR_SHOOT2,
    SPR_GOOD_DR_DIE1,SPR_GOOD_DR_DIE2,SPR_GOOD_DR_DIE3,SPR_GOOD_DR_DEAD,
    SPR_PARTY_O1,SPR_PARTY_O2,SPR_PARTY_O3,SPR_PARTY_O4,
    
    SPR_ALP_W1,SPR_ALP_W2,SPR_ALP_W3,SPR_ALP_W4,
    SPR_ALP_SHOOT,SPR_FIRE1,SPR_FIRE2,
    SPR_ALP_DIE1,SPR_ALP_DIE2,SPR_ALP_DIE3,SPR_ALP_DIE4,
    SPR_ALP_DIE5,SPR_ALP_DEAD,
    
    SPR_MECHA_W1,SPR_MECHA_W2,SPR_MECHA_W3,SPR_MECHA_W4,
    SPR_MECHA_SHOOT1,SPR_MECHA_SHOOT2,SPR_MECHA_SHOOT3,SPR_MECHA_DEAD,
    SPR_MECHA_DIE1,SPR_MECHA_DIE2,SPR_MECHA_DIE3,
    
    SPR_GROFAZ_W1,SPR_GROFAZ_W2,SPR_GROFAZ_W3,SPR_GROFAZ_W4,
    SPR_GROFAZ_SHOOT1,SPR_GROFAZ_SHOOT2,SPR_GROFAZ_SHOOT3,SPR_GROFAZ_DEAD,
    SPR_GROFAZ_DIE1,SPR_GROFAZ_DIE2,SPR_GROFAZ_DIE3,SPR_GROFAZ_DIE4,
    SPR_GROFAZ_DIE5,SPR_GROFAZ_DIE6,SPR_GROFAZ_DIE7,
    
    SPR_HERMANN_W1,SPR_HERMANN_W2,SPR_HERMANN_W3,SPR_HERMANN_W4,
    SPR_HERMANN_SHOOT1,SPR_HERMANN_SHOOT2,
    SPR_HERMANN_DIE1,SPR_HERMANN_DIE2,SPR_HERMANN_DIE3,SPR_HERMANN_DEAD,
    
    SPR_ROCKET_1,SPR_ROCKET_2,SPR_ROCKET_3,SPR_ROCKET_4,
    SPR_ROCKET_5,SPR_ROCKET_6,SPR_ROCKET_7,SPR_ROCKET_8,
    
    SPR_SMOKE_1,SPR_SMOKE_2,SPR_SMOKE_3,SPR_SMOKE_4,
    SPR_BOOM_1,SPR_BOOM_2,SPR_BOOM_3,

    SPR_GRETEL_W1,SPR_GRETEL_W2,SPR_GRETEL_W3,SPR_GRETEL_W4,
    SPR_GRETEL_SHOOT1,SPR_GRETEL_SHOOT2,SPR_GRETEL_SHOOT3,SPR_GRETEL_DEAD,
    SPR_GRETEL_DIE1,SPR_GRETEL_DIE2,SPR_GRETEL_DIE3,
   
    SPR_FETTGEISCHT_W1,SPR_FETTGEISCHT_W2,SPR_FETTGEISCHT_W3,SPR_FETTGEISCHT_W4,
    SPR_FETTGEISCHT_SHOOT1,SPR_FETTGEISCHT_SHOOT2,SPR_FETTGEISCHT_SHOOT3,SPR_FETTGEISCHT_SHOOT4,
    SPR_FETTGEISCHT_DIE1,SPR_FETTGEISCHT_DIE2,SPR_FETTGEISCHT_DIE3,SPR_FETTGEISCHT_DEAD,
   
    SPR_SELF1_W1,SPR_SELF_W2,SPR_SELF_W3,SPR_SELF_W4,
    SPR_SELF_JUMP1,SPR_SELF_JUMP2,SPR_SELF_JUMP3,SPR_SELF_JUMP4,
   
    SPR_KNIFEREADY,SPR_KNIFEATK1,SPR_KNIFEATK2,SPR_KNIFEATK3,
    SPR_KNIFEATK4,
   
    SPR_PISTOLREADY,SPR_PISTOLATK1,SPR_PISTOLATK2,SPR_PISTOLATK3,
    SPR_PISTOLATK4,
   
    SPR_MACHINEGUNREADY,SPR_MACHINEGUNATK1,SPR_MACHINEGUNATK2,SPR_MACHINEGUNATK3,
    SPR_MACHINEGUNATK4,
   
    SPR_CHAINREADY,SPR_CHAINATK1,SPR_CHAINATK2,SPR_CHAINATK3,
    SPR_CHAINATK4,

}