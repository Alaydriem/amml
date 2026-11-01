entry "IntOption.set" {
  kind      = "function"
  msvc_name = "?set@IntOption@@QEAAXH_N@Z"
  notes     = "void IntOption::set(int value, bool saveOptionChange): 0x91cda0 in 26.51, 0x9bf8a0 in 26.60. rcx = the IntOption, edx = the value, r8b = the save flag; it returns void. It returns at once when the value does not change (`cmp [rcx+18h],edx; je`); else it clamps the value (to the nearest of the possible values at +28h/+30h when mClampToRange at +20h is set, or to VALUE_MIN at +14h and VALUE_MAX at +10h), stores it in mValue (+18h), calls the value-changed notify on mImpl (Option+8), and when the flag is set calls the request-save std::function at mImpl+298h. A direct write to mValue skips the notify, so a mod calls this function. Recipe 0 is the body after the stack cookie: `cmp [rcx+disp],edx; je rel32; mov ebx,r8d; mov rsi,rcx; cmp byte ptr [rcx+disp],imm; jne; mov rcx,[rsi+disp]; mov rax,[rsi+disp]; cmp rax,rcx`, then `function_start_of`. Recipe 1 is the call in IOptionRegistry::setDifficulty (OptionRegistry.vtable slot 93 in both builds): `mov r8d,2; call [guard]` (get(2), the difficulty option), `mov rcx,[rsp+..]; mov rax,[rcx+8]; cmp qword ptr [rax+238h],0; jne; mov edx,esi; mov r8b,1; call`, read as a branch target at offset 36. One byte is kept: the option id 2, because twenty other option setters have the same code with other ids. The two recipes are in two functions. Every other immediate, displacement and branch byte is a wildcard. Both are unique in both exes and agree. Check: function_start. Not hooked: a mod calls it. What changed in 26.60: the address only; the body is the same."

  recipe {
    step "pattern" {
      bytes = "39 51 ?? 0F 84 ?? ?? ?? ?? 44 89 C3 48 89 CE 80 79 ?? ?? 75 ?? 48 8B 4E ?? 48 8B 46 ?? 48 39 C8"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes      = "41 B8 02 ?? ?? ?? FF 15 ?? ?? ?? ?? 48 8B 4C 24 ?? 48 8B 41 ?? 48 83 B8 ?? ?? ?? ?? ?? 75 ?? 89 F2 41 B0 ?? E8"
      offset     = 36
      literal_ok = [
        "2: mov r8d,2, the option id of the difficulty option: twenty other option setters have the same code with other ids (26.51 and 26.60)",
      ]
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
