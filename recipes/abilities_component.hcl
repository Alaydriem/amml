entry "AbilitiesComponent.mPlayerPermissions" {
  kind     = "field"
  notes    = "Byte offset of the player permission level (PlayerPermissionLevel: Visitor 0, Member 1, Operator 2, Custom 3) in AbilitiesComponent: 1 on 26.63 and 70.26 (LayeredAbilities::mPermissions: byte 0 the command permission \"permissionsLevel\", byte 1 the player permission \"playerPermissionsLevel\"). Read it as a u8 at this offset from the component pointer that Player.getAbilitiesComponent writes (70.x); on 26.63 Player.getPlayerPermissionLevel returns the same byte. One recipe, a pattern_any with one alternative per build shape; each alternative matches once on its own build and 0 times on the other. Alternative 0 (26.63) is the end of Player::getPlayerPermissionLevel: `and edx,7Fh; imul rcx,rdx,<pool element size>; movzx eax,byte ptr [rax+rcx+disp8]; add rsp,..; ret`; the movzx ModRM 44h stays, because it fixes the disp8 form (Player::getCommandPermissionLevel ends with ModRM 04h and no displacement). Alternative 1 (70.x) is a player function that switches on the permission level, from the getter call: `call getter; mov rdi,[rsp+..]; movzx eax,byte ptr [rdi+disp8]; cmp eax,imm; je; cmp eax,imm; jne` (it tests Operator 2, then Custom 3); the isOperator wrapper is not used because its read is a `cmp byte ptr [mem],imm`, whose memory operand has another index than the movzx of alternative 0. Both put the byte access at offset 10 and read its displacement of the byte access. Every other immediate, displacement, branch and register byte is a wildcard. The range check is a window for a small offset in the component. No build gate: the member has the same meaning on both builds. Evidence on 70.26: (1) the 26.63 isOperator wrapper (`call getPlayerPermissionLevel; cmp al,2; setge`) is found once on 70.26 and compares byte +1 of the component with 2; (2) the 26.63 server caller (`call getPlayerPermissionLevel; cmp al,2; jl`) is found on 70.26 as `call getter; mov rax,[rsp+..]; cmp byte ptr [rax+1],2; jl`. What changed in 6afa4e47-dadc375797a3d31a: nothing in the component; the reads moved into the callers. Class: new entry (replacement for Player.getPlayerPermissionLevel)."

  recipe {
    step "pattern_any" {
      patterns = [
        "83 ?? ?? ?? 69 ?? ?? ?? ?? ?? 0F B6 44 ?? ?? 48 83 C4 ?? C3",
        "E8 ?? ?? ?? ?? ?? 8B ?? 24 ?? 0F B6 ?? ?? 83 F8 ?? 74 ?? 83 F8 ?? 0F 85",
      ]
      offset     = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 63
  }
}
