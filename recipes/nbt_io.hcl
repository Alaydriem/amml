entry "NbtIo.read" {
  kind      = "function"
  msvc_name = "?read@NbtIo@@YA?AV?$Result@V?$unique_ptr@VCompoundTag@@U?$default_delete@VCompoundTag@@@std@@@std@@Verror_code@2@@Bedrock@@AEAVIDataInput@@@Z"
  notes     = "Bedrock::Result<std::unique_ptr<CompoundTag>> NbtIo::read(IDataInput&): a free function, rcx = the result buffer (hidden return pointer), rdx = the input. It reads the root named tag into a temporary std::string name and returns the compound. On 26.51 the body writes one pointer (the unique_ptr) into the result buffer, not a Result with an error code, and for bytes that do not read it gives an empty compound, never null (seen in the Task 4 gate). A caller builds a StringByteInput inline and passes its address in rdx. Recipe 0 is the body head (`mov rsi,rcx; xorps xmm0,xmm0; movaps [rbp+..],xmm0; mov qword ptr [rbp+..],0; mov qword ptr [rbp+..],0Fh; lea rcx,[rbp+..]; lea r8,[rbp+..]; call`), then function_start_of. Recipe 1 is the call after an inline StringByteInput (`lea rcx,[rip+StringByteInput vtable]; ..; lea rcx,[rbp+..]; lea rdx,[rbp+..]; call`), the same pattern_any as StringByteInput.vtable recipe 1, read at offset 53. Check function_start. On 26.60 the body also writes one pointer to the result buffer (`mov [rsi],rbx; mov rax,rsi`, rsi = rcx), so the 8-byte sret buffer of the mod holds on both builds. What changed in 6ad79ee4-17d51891a1119914 (26.60): the 26.51 site lost its `mov r14d,[rbx]` prefix and the call has a nop after it. Would have survived with a better recipe."

  recipe {
    step "pattern" { bytes = "48 89 CE 0F 57 C0 0F 29 45 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 C7 45 ?? ?? ?? ?? ?? 48 8D 4D ?? 4C 8D 45 ?? E8" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "48 8D 0D ?? ?? ?? ?? 48 89 8D ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 B5 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? E8 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 0F 57 F6",
        "48 8D 0D ?? ?? ?? ?? 48 89 8D ?? ?? ?? ?? 48 C7 85 ?? ?? ?? ?? ?? ?? ?? ?? 48 89 B5 ?? ?? ?? ?? 48 89 85 ?? ?? ?? ?? 48 8D 8D ?? ?? ?? ?? 48 8D 95 ?? ?? ?? ?? E8 ?? ?? ?? ?? 90 48 8B 85 ?? ?? ?? ??",
      ]
      offset = 53
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
