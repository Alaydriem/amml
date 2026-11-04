entry "ClientWaypoint.mColor" {
  kind     = "field"
  notes    = "The offset of ClientWaypoint::mColor (std::optional<mce::Color>: four floats, then the engaged byte, 14h bytes) in ClientWaypoint: 18h on 26.60. HudLocatorBarRenderer.render (0x5745690 on 26.60) reads the colour of each waypoint it draws: it takes the colour when the optional is engaged and a default colour in .rdata when not (`mov rax,[rax]; cmp byte ptr [rax+..],imm; lea rcx,[rax+..]; lea rdx,[rip+..]; cmove rcx,rdx; movups xmm0,[rcx]`). One recipe: that pattern, read as the displacement of the lea of the colour. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 8B 00 80 78 ?? ?? 48 8D 48 ?? 48 8D 15 ?? ?? ?? ?? 48 0F 44 CA 0F 10 01"
      offset = 7
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
  notes    = "The offset of the engaged byte of ClientWaypoint::mColor (std::optional<mce::Color>) in ClientWaypoint: 28h on 26.60 (the colour at 18h, 16 bytes, then the byte). HudLocatorBarRenderer.render (0x5745690 on 26.60) tests it before it reads the colour (`mov rax,[rax]; cmp byte ptr [rax+..],imm; lea rcx,[rax+..]; lea rdx,[rip+..]; cmove rcx,rdx; movups xmm0,[rcx]`). One recipe: that pattern, read as the displacement of the cmp. It shares its bytes with ClientWaypoint.mColor. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 8B 00 80 78 ?? ?? 48 8D 48 ?? 48 8D 15 ?? ?? ?? ?? 48 0F 44 CA 0F 10 01"
      offset = 3
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
  notes    = "The offset of the visible flag (a bool) in ClientWaypoint: 60h on 26.60. HudLocatorBarRenderer.render (0x5745690 on 26.60) walks the sorted waypoint pointers and skips a waypoint whose flag is 0, clearing a byte of it (`mov byte ptr [rax+..],0; add r13,8; ...; mov rax,[r13]; cmp byte ptr [rax+..],imm; je; movss xmm12,[rax+..]; movss xmm0,[rax+..]`; the two loads read the position). One recipe: the load of the pointer, the test and the position loads, read as the displacement of the cmp. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "49 8B 45 ?? 80 78 ?? ?? 74 ?? F3 44 0F 10 60 ?? F3 0F 10 40 ??"
      offset = 4
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
