entry "FontHandle.dtor" {
  kind     = "function"
  notes    = "FontHandle::~FontHandle() without freeing the object (the non-deleting destructor): rcx = the FontHandle, no result. It stores the FontHandle vtable (`lea rax,[rip+..]; mov [rcx],rax`), calls the EnableNonOwnerReferences cleanup (`lea rdx,[rip+..]; call`), then releases the shared_ptr<Font> control block (+38h) and the NonOwnerPointer (+20h). The caller of getFontHandle destroys its by-value result with it (extractNameTags: `lea rcx,[rbp-30h]; call dtor`). Recipe 0 is the head of the body (`push rbp; push rsi; push rdi; sub rsp,imm; lea rbp,[rsp+..]; mov qword ptr [rbp-8],-2; mov rsi,rcx; lea rax,[rip+..]; mov [rcx],rax; lea rdx,[rip+..]; call`), then function_start_of. Recipe 1 is the call in the text-width function (0x15632c0): `cvtsi2ss xmm0,eax; movss [rsi+..],xmm0; lea rcx,[rbp+..]; call`, read as a branch target. They are in two functions. Every register, displacement, immediate and branch byte is a wildcard. In 6ad79ee4-17d51891a1119914 (26.60) the destructor is at 0xE7F9E0; both recipes derived on the cold run. Class: survived."

  recipe {
    step "pattern" {
      bytes  = "55 56 57 48 83 EC ?? 48 8D 6C 24 ?? 48 C7 45 ?? ?? ?? ?? ?? 48 89 CE 48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 15 ?? ?? ?? ?? E8"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "F3 0F 2A C0 F3 0F 11 86 ?? ?? ?? ?? 48 8D 4D ?? E8"
      offset = 16
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}

}

entry "FontHandle.size" {
  kind     = "field"
  notes    = "sizeof(FontHandle): 0x50 (80) in 26.51 (EnableNonOwnerReferences 0x18: vtable and a shared_ptr control; mFontRepository 0x18, mDefaultFont 0x30, mFontId 0x40, mIsDummyHandle 0x48, padded to 8). It is the size of the buffer that holds a FontHandle returned by value, and the size check of the FontHandle mirror of the mod (the members and the FontRepository font vector are read from the 26.51 header layout, not from entries). The body of the ClientInstance virtual getFontHandle copies the members into the result: `mov [rsi+18h],rcx; mov [rsi+20h],rax; ...; mov [rsi+30h],rcx; mov [rsi+38h],rax; movzx eax,byte ptr [rbx+118h]; mov [rsi+48h],al; mov rax,[rbx+110h]; mov [rsi+40h],rax`. The ClientInstance has three look-alike getters in a row (slots 89 to 91) with the same stores from other source members; the source member offset 118h tells them apart, so its two bytes are kept (literal_ok). Recipe 0 reads the displacement of the byte store of mIsDummyHandle (0x48) and adds 8 (the last member is one byte, padded to 8). Recipe 1 reads the displacement of the store of mFontId (0x40) and adds 16 (8 for the id, 8 for the padded flag). They read two different stores of one function. In 6ad79ee4-17d51891a1119914 (26.60) the value is still 0x50: the member copy stores the same members at +30h, +38h, +40h and +48h. Both recipes derived on the cold run. Class: survived."

  recipe {
    step "pattern" {
      bytes  = "0F B6 83 18 01 ?? ?? 88 46 ?? 48 8B 83 ?? ?? ?? ?? 48 89 46 ??"
      offset = 7
      literal_ok = ["3: member offset 118h of the source object: the two sibling getters read other members", "4: second byte of the same member offset"]
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" {
      value = 8
    }
  }

  recipe {
    step "pattern" {
      bytes  = "0F B6 83 18 01 ?? ?? 88 46 ?? 48 8B 83 ?? ?? ?? ?? 48 89 46 ??"
      offset = 17
      literal_ok = ["3: member offset 118h of the source object: the two sibling getters read other members", "4: second byte of the same member offset"]
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" {
      value = 16
    }
  }

  check "range" {
    min = 48
    max = 160
  }

}

entry "FontHandle.mDefaultFont" {
  kind     = "field"
  notes    = "The offset of FontHandle::mDefaultFont (shared_ptr<Font>: the Font at +0, the control block at +8) in FontHandle: 30h on 26.60 (mFontRepository 18h, mDefaultFont 30h, mFontId 40h, mIsDummyHandle 48h). Recipe 0 is the member copy in the body of the ClientInstance virtual getFontHandle (0x65ab760 on 26.60; the anchor of FontHandle.size): `mov rcx,[rbx+..]; mov [rsi+..],rcx; mov [rsi+..],rax; movzx eax,byte ptr [rbx+118h]`, read as the displacement of the store of the Font pointer. The source member offset 118h tells this getter from its two look-alike siblings (slots 89 to 91), so its two bytes are kept (literal_ok), as in FontHandle.size. Recipe 1 is FontHandle.dtor (0xe7f9e0 on 26.60), which first releases the control block of mDefaultFont (`nop; mov rdi,[rsi+..]; test rdi,rdi; je; lock dec dword ptr [rdi+..]; jne`, after the call that cleans up the base), read as the displacement of the load, then `add -8`. They are in two functions. Every other displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "48 8B 8B ?? ?? ?? ?? 48 89 4E ?? 48 89 46 ?? 0F B6 83 18 01 ?? ??"
      offset = 7
      literal_ok = ["18: member offset 118h of the source object: the two sibling getters read other members", "19: second byte of the same member offset"]
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "FontHandle.dtor" }
    step "pattern" {
      bytes  = "90 48 8B 7E ?? 48 85 FF 74 ?? F0 FF 4F ?? 75"
      offset = 1
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" { value = -8 }
  }

  check "range" {
    min = 16
    max = 96
  }
}
