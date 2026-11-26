entry "JsonValue.toStyledString" {
  kind     = "function"
  notes    = "`std::string Json::Value::toStyledString() const`: rcx = the Json::Value (this), rdx = the hidden out pointer to an uninitialised 32-byte std::string; rax returns rdx. The body builds a Json::StyledWriter in its frame, calls StyledWriter::write(root) with the out pointer in rdx and the value in r8, then destroys the writer. The string is allocated with the game's global operator new; callers free it with the game's sized operator delete, as the skin size function does after it reads the length. 57 direct callers on 26.63, 53 on 70.26; among them the skin size function (after the animated image loop it serialises SerializedSkinImpl::mGeometryData with this call to count its bytes) and the connection request writer (the only loader of the string SkinResourcePatch). The body is a small wrapper and no body pattern is unique (on 70.26 the prologue up to the writer constructor call matches three functions), so both recipes are call sites read as the call target. Recipe 0 is the end of a loop in a caller (`add rdi,imm8; cmp rdi,r15; jne; lea rcx,[rbp+..]; lea rdx,[rbp+..]; call`). Recipe 1 is a caller that picks a 16-byte constant and joins (`movups xmm0,[rip+..]; movaps [rbp+..],xmm0; jmp; lea rcx,[rbp+..]; lea rdx,[rbp+..]; call`). The two anchors are in two functions and share no pattern bytes; both end at the call. Every displacement, immediate and branch byte is a wildcard. function_start confirms the start (neither recipe ends with function_start_of). What changed in 6afa4e47-dadc375797a3d31a (70.26): the function is built with another code shape (a frame pointer with push rbp, the writer destructor out of line), so the old body recipe (the stack cookie form) does not match; the skin size function now calls a thread-owner check before each member read, so the old call-site recipe there does not match. Class: real code change (shape); the function and its signature are the same. Evidence on 70.26: (1) call sites drafted from 26.63: of the 57 old call sites, 11 drafted sites (five instructions up to the call) are unique on both builds; ten of them (the two recipe sites among them) call this function on 70.26, and the eleventh, a generic `call [guard]; nop; lea; lea; mov rdx,rdi; call` shape, lands in an unrelated function; the skin size function (the same animated image loop, then the mGeometryData member) and the connection request writer (the SkinResourcePatch loader, the same place after the string free) call it at the old place. (2) The body has the old shape (writer built in the frame, write with rdx the out pointer and r8 the value, writer destroyed, the out pointer returned), and its write callee matches the old StyledWriter::write step for step (document_ at +20h cleared, addChildValues_ at +68h set to 0, indentString_ at +40h cleared, writeValue, a newline appended, document_ copied to the out string)."

  recipe {
    step "pattern" {
      bytes  = "48 83 C7 ?? 4C 39 FF 75 ?? 48 8D 4D ?? 48 8D 55 ?? E8 ?? ?? ?? ??"
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "0F 10 05 ?? ?? ?? ?? 0F 29 45 ?? EB ?? 48 8D 4D ?? 48 8D 55 ?? E8 ?? ?? ?? ??"
      offset = 21
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
