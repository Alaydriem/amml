entry "MapPolicy.get" {
  kind     = "vslot"
  notes    = "Slot of the MapPolicy virtual that returns the tint for a block and position (MapPolicy::get): slot 1 on 26.51 and 26.60 (`mov rcx,[rax]; mov r10,[rcx+8]` on the policy that BiomeColorSampling.getMapPolicy returns, then `call [guard]` through rax = r10). Function A is a map-colour function found by its own anchor, `lea r14,[rbx+..]; mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,r14; call [guard]` (unique on 26.51 and 26.60; a byte-identical twin of A on 26.60 has the first two instructions swapped). Function B is a second map-colour function found by `mov [rbp+..],eax; mov rdi,[rdi+..]; mov rcx,rbx; call`: a pattern_any with one alternative per shape, because the store is disp32 on 26.51 and disp8 on 26.60 (the frame changed). Both anchors end at a call. The steps after the anchor scan only the function that holds it. Every displacement, immediate and branch byte is a wildcard. Recipe 0: function A, then `mov rcx,[rax]; mov r10,[rcx+..]; lea rdx,[rbp+..]`, operand 1 displacement as a slot. Recipe 1: function B, then `mov rcx,[rax]; mov r10,[rcx+..]; lea rdx,[rbp+..]; lea r9,..`, the same. They are in two functions. Check range 0..8. Evidence on 26.60: the virtual call on the getMapPolicy result, in function A and in function B, reads +8. What changed in 6ad79ee4-17d51891a1119914 (26.60): every call is followed by a nop, so each old pattern that ran past a call stopped matching; the old scripting map-colour function became a byte-identical twin of function A (same registers), so a pattern in it is no longer unique; the colour is now read with one movups/mulps instead of four movss. Class: real code change (call-site nops, register allocation, vectorised colour); patterns that end at a call would have survived the nops. Grade: two recipes in two functions plus range."

  recipe {
    step "pattern" { bytes = "4C 8D 73 ?? 48 8B 4B ?? 48 8B 01 48 8B 40 ?? 4C 89 F2 FF 15" }
    step "pattern" {
      bytes  = "48 8B 08 4C 8B 51 ?? 48 8D 55"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  recipe {
    step "pattern_any" {
      patterns = ["89 85 ?? ?? ?? ?? 48 8B 7F ?? 48 89 D9 E8", "89 45 ?? 48 8B 7F ?? 48 89 D9 E8"]
    }
    step "pattern" {
      bytes  = "48 8B 08 4C 8B 51 ?? 48 8D 55 ?? 4C 8D"
      offset = 3
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "displacement_to_slot" {}
  }

  check "range" {
    min = 0
    max = 8
  }
}
