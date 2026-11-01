entry "JsonValue.toStyledString" {
  kind     = "function"
  notes    = "`std::string Json::Value::toStyledString() const`: 0xe932900 on 26.60, 0xe01ef80 on 26.51 (the same 9Eh-byte body on both builds; 57 direct callers on 26.60). rcx = the Json::Value (this), rdx = the hidden out pointer to an uninitialised 32-byte std::string; rax returns rdx. The body builds a Json::StyledWriter on the stack (the writer vtable, childValues_ vector, document_ and indentString_ strings), calls StyledWriter::write(root) with the out pointer in rdx and the value in r8, then destroys the writer. The string is allocated with the game's global operator new; callers free it with the game's sized operator delete (0xe934b00 on 26.60, as the skin size function 0x298f4e0 does after it reads the length). Found from that skin size function, which serialises SerializedSkinImpl::mGeometryData with this call to count its bytes. Recipe 0 is the body after the stack cookie (`mov rdi,rdx; mov rbx,rcx; mov [rsp+..],rdx; lea rcx,[rsp+..]; call` (the writer constructor)), then function_start_of. Recipe 1 is the call in the skin size function (0x298f4e0 on 26.60, 0x2650180 on 26.51: the end of the animated image loop, then `add rcx,mGeometryData; lea rdx,[rsp+..]; call`), read as a branch target. The two anchors are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. function_start confirms the start."

  recipe {
    step "pattern" { bytes = "48 8B FA 48 8B D9 48 89 54 24 ?? 48 8D 4C 24 ?? E8" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 39 D0 75 ?? 48 81 C1 ?? ?? ?? ?? 48 8D 54 24 ?? E8"
      offset = 17
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
