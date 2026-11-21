entry "MapPolicy.get" {
  kind     = "vslot"
  notes    = "Slot of the MapPolicy virtual that returns the tint for a block and position (MapPolicy::get): slot 1 on 26.51 and 26.60 (`mov rcx,[rax]; mov r10,[rcx+8]` on the policy that BiomeColorSampling.getMapPolicy returns, then `call [guard]` through rax = r10). Function A is a map-colour function found by its own anchor, `lea r14,[rbx+..]; mov rcx,[rbx+..]; mov rax,[rcx]; mov rax,[rax+..]; mov rdx,r14; call [guard]` (unique on 26.51 and 26.60; a byte-identical twin of A on 26.60 has the first two instructions swapped). Function B is a second map-colour function found by `mov [rbp+..],eax; mov rdi,[rdi+..]; mov rcx,rbx; call`: a pattern_any with one alternative per shape, because the store is disp32 on 26.51 and disp8 on 26.60 (the frame changed). Both anchors end at a call. The steps after the anchor scan only the function that holds it. Every displacement, immediate and branch byte is a wildcard. Recipe 0: function A, then `mov rcx,[rax]; mov r10,[rcx+..]; lea rdx,[rbp+..]`, operand 1 displacement as a slot. Recipe 1: function B, then `mov rcx,[rax]; mov r10,[rcx+..]; lea rdx,[rbp+..]; lea r9,..`, the same. They are in two functions. Check range 0..8. Evidence on 26.60: the virtual call on the getMapPolicy result, in function A and in function B, reads +8. What changed in 6ad79ee4-17d51891a1119914 (26.60): every call is followed by a nop, so each old pattern that ran past a call stopped matching; the old scripting map-colour function became a byte-identical twin of function A (same registers), so a pattern in it is no longer unique; the colour is now read with one movups/mulps instead of four movss. Class: real code change (call-site nops, register allocation, vectorised colour); patterns that end at a call would have survived the nops. Grade: two recipes in two functions plus range. What changed in 1.26.70.26 (6afa4e47-dadc375797a3d31a): the map-colour code of function A (and of the old functions like it) moved into one small function that the 7 callers now call (it gets the BlockMapColorComponent, calls the getMapPolicy function, then `mov rcx,[rax]; mov r10,[rcx+8]; lea rdx,[rsp+..]; mov rcx,rax; mov r8,rbx; mov r9,rdi; mov rax,r10; call [guard]; movups xmm0,[r14]; mulps xmm0,[rsp+..]`), and function B is no longer a function of its own with a call of getMapPolicy; the one remaining direct call of getMapPolicy is in that small function. Recipe 0 is a pattern_any (its first alternative now runs to the call after the nop, because the short anchor of 26.63 also matches a twin function on 70.26) whose second alternative anchors on the call of getMapPolicy in it (`movzx ecx,byte ptr [rax+10h]; mov r14,rax; call`), then the same in-function pattern for the vcall with its operand form (`lea rdx,[rsp+..]` on 70.26). Recipe 1 has a third alternative that anchors on the colour multiply after the vcall in the same function (`movups xmm0,[r14]; mulps xmm0,[rsp+..]; movups [rsi],xmm0`); the two recipes of 70.26 therefore lie in one function and are two anchors on two ends of it. Evidence from the old function: (1) the vcall block of function A (the load of the component byte, the call of getMapPolicy, the slot 1 call with rdx = a vec4 result, r8, r9, then the multiply of the component colour by that vec4) is, instruction for instruction, the body of the small function of 70.26 (the only function in the exe with that block after a call of the function that the other repair found as getMapPolicy); (2) that vcall passes the same arguments as in A and B (rdx result, r8, r9) and the function has 7 direct callers on 70.26, taking the place of the inlined copies in A and in the old callers of getMapPolicy. Class: real code change (the code was factored into a function), repaired by a second code shape."

  recipe {
    step "pattern_any" {
      patterns = [
        "4C 8D 73 ?? 48 8B 4B ?? 48 8B 01 48 8B 40 ?? 4C 89 F2 FF 15 ?? ?? ?? ?? 90 49 89 C4 48 8B 5B ?? 48 89 C1 E8",
        "0F B6 48 ?? 49 89 ?? E8",
      ]
    }
    step "pattern_any" {
      patterns = [
        "48 8B 08 4C 8B 51 ?? 48 8D 55",
        "48 8B 08 4C 8B 51 ?? 48 8D 54 24 ?? 48 89 C1",
      ]
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
      patterns = [
        "89 85 ?? ?? ?? ?? 48 8B 7F ?? 48 89 D9 E8",
        "89 45 ?? 48 8B 7F ?? 48 89 D9 E8",
        "41 0F 10 06 0F 59 44 24 ?? 0F 11 06",
      ]
    }
    step "pattern_any" {
      patterns = [
        "48 8B 08 4C 8B 51 ?? 48 8D 55 ?? 4C 8D",
        "48 8B 08 4C 8B 51 ?? 48 8D 54 24 ?? 48 89 C1",
      ]
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
