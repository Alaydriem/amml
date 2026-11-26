entry "ClientWaypoint.mColor" {
  kind     = "field"
  notes    = "The offset of ClientWaypoint::mColor (std::optional<mce::Color>: four floats, then the engaged byte, 14h bytes) in ClientWaypoint: 18h on 26.60. HudLocatorBarRenderer.render (0x5745690 on 26.60) reads the colour of each waypoint it draws: it takes the colour when the optional is engaged and a default colour in .rdata when not (`mov rax,[rax]; cmp byte ptr [rax+..],imm; lea rcx,[rax+..]; lea rdx,[rip+..]; cmove rcx,rdx; movups xmm0,[rcx]`). One recipe: that pattern, read as the displacement of the lea of the colour. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked. What changed in 6afa4e47-dadc375797a3d31a (70.26): in HudLocatorBarRenderer.render (slot 16 of HudLocatorBarRenderer.vtable on both builds, with the same neighbours) the compiler reordered the colour test, so the recipe has a second alternative (one alternative per shape; each matches once on its own build and not on the other). Value 18h on 6afa4e47-dadc375797a3d31a. Would have survived with a better recipe. Evidence on 70.26: the vtable of HudLocatorBarRenderer lines up around slot 16 (slots 9, 16, 18 and 20 are functions and the shared-thunk slots are in the same places on both builds), and the pattern drafted from the 26.63 body reads the same member in the function of that slot (the shape differs only by the order of the cmp and the lea)."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8B 00 80 78 ?? ?? 48 8D 48 ?? 48 8D 15 ?? ?? ?? ?? 48 0F 44 CA 0F 10 01",
        "?? ?? ?? ?? ?? ?? ?? 48 8D 48 ?? 80 78 ?? ?? 48 8D 15 ?? ?? ?? ?? 48 0F 44 CA 0F 10 01",
      ]
      offset   = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 64
  }
}

entry "ClientWaypoint.mColorEngaged" {
  kind     = "field"
  notes    = "The offset of the engaged byte of ClientWaypoint::mColor (std::optional<mce::Color>) in ClientWaypoint: 28h on 26.60 (the colour at 18h, 16 bytes, then the byte). HudLocatorBarRenderer.render (0x5745690 on 26.60) tests it before it reads the colour (`mov rax,[rax]; cmp byte ptr [rax+..],imm; lea rcx,[rax+..]; lea rdx,[rip+..]; cmove rcx,rdx; movups xmm0,[rcx]`). One recipe: that pattern, read as the displacement of the cmp. It shares its bytes with ClientWaypoint.mColor. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked. What changed in 6afa4e47-dadc375797a3d31a (70.26): the compiler moved the `lea` of the colour before the `cmp` of the engaged byte and loads the waypoint through r13, so the recipe is a pattern_any with one alternative per shape (the old shape has one wildcard byte in front so that the cmp sits at offset 4 in both). Value 28h on 6afa4e47-dadc375797a3d31a. Would have survived with a better recipe. Evidence on 70.26: the vtable of HudLocatorBarRenderer lines up around slot 16 (the render slot) on both builds, and the pattern drafted from the 26.63 body reads the member in the function of that slot, next to the colour load that ClientWaypoint.mColor reads at 18h (16 bytes) before it."

  recipe {
    step "pattern_any" {
      patterns = [
        "?? 48 8B 00 80 78 ?? ?? 48 8D 48 ?? 48 8D 15 ?? ?? ?? ?? 48 0F 44 CA 0F 10 01",
        "48 8D 48 ?? 80 78 ?? ?? 48 8D 15 ?? ?? ?? ?? 48 0F 44 CA 0F 10 01",
      ]
      offset   = 4
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 80
  }
}

entry "ClientWaypoint.mVisible" {
  kind     = "field"
  notes    = "The offset of the visible flag (a bool) in ClientWaypoint: 60h on 26.60. HudLocatorBarRenderer.render (0x5745690 on 26.60) walks the sorted waypoint pointers and skips a waypoint whose flag is 0, clearing a byte of it (`mov byte ptr [rax+..],0; add r13,8; ...; mov rax,[r13]; cmp byte ptr [rax+..],imm; je; movss xmm12,[rax+..]; movss xmm0,[rax+..]`; the two loads read the position). One recipe: the load of the pointer, the test and the position loads, read as the displacement of the cmp. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked. What changed in 6afa4e47-dadc375797a3d31a (70.26): the first position load is `movss xmm7,[rax+..]` (no REX prefix) instead of `movss xmm12,[rax+..]`, so recipe 0 is a pattern_any with one alternative per shape. Value 60h on 6afa4e47-dadc375797a3d31a. Would have survived with a better recipe. Evidence on 70.26: the vtable of HudLocatorBarRenderer lines up around slot 16 (the render slot) on both builds, and the pattern drafted from the 26.63 body, the waypoint loop of that function, reads the member."

  recipe {
    step "pattern_any" {
      patterns = [
        "49 8B 45 ?? 80 78 ?? ?? 74 ?? F3 44 0F 10 ?? ?? F3 0F 10 40 ??",
        "49 8B 45 ?? 80 78 ?? ?? 74 ?? F3 0F 10 ?? ?? F3 0F 10 40 ??",
      ]
      offset   = 4
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 48
    max = 128
  }
}
