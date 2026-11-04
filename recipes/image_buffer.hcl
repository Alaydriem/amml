entry "ImageBuffer.ctorFromDescription" {
  kind     = "function"
  notes    = "`cg::ImageBuffer::ImageBuffer(cg::ImageDescription const&)`: 0x5a4bb80 on 26.60, 0x5964d10 on 26.51. rcx = the 30h-byte ImageBuffer to build, rdx = the 20-byte ImageDescription; it returns rcx. ImageBuffer layout: +0 the blob deleter function, +8 the data pointer, +10h the byte size, +18h the copied ImageDescription (+28h is its depth). ImageDescription: +0 width, +4 height, +8 mce::TextureFormat (RGBA8 is 1Ch), +0Ch colour space, +0Dh image type, +0Eh alpha usage, +10h array size or depth. The body stores the deleter and depth 1, copies the description, and allocates bytes-per-pixel * width * height * depth (times 6 for a cube image type). Recipe 0 is the copy of the description (`mov eax,[rsi+10h]; mov rcx,[rbp+..]; mov [rcx+28h],eax; movups xmm0,[rsi]; movups [rcx+18h],xmm0; mov edx,[rsi]; mov r8d,[rsi+4]; mov eax,edx; or eax,r8d`), then function_start_of; it stops before the branch, which is short on 26.60 and near on 26.51. Recipe 1 is the call in TextureGroup::addEmptyTexture (0x54239a0 on 26.60), which fills a description on the stack and builds the buffer from it (`mov dword ptr [rbp+..],imm; mov byte ptr [rbp+..],imm; mov dword ptr [rbp+..],imm; mov byte ptr [rbp+..],imm; lea rbx,[rbp+..]; lea rdx,[rbp+..]; mov rcx,rbx; call`), read as a branch target. The two anchors are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard."

  recipe {
    step "pattern" { bytes = "8B 46 ?? 48 8B 4D ?? 89 41 ?? 0F 10 06 0F 11 41 ?? 8B 16 44 8B 46 ?? 89 D0 44 09 C0" }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "C7 45 ?? ?? ?? ?? ?? C6 45 ?? ?? C7 45 ?? ?? ?? ?? ?? C6 45 ?? ?? 48 8D 5D ?? 48 8D 55 ?? 48 89 D9 E8"
      offset = 33
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "ImageBuffer.size" {
  kind     = "field"
  notes    = "sizeof(cg::ImageBuffer): 30h on 26.60 (the mce::Blob at +0: deleter, data pointer, byte size; the 20-byte ImageDescription at +18h, padded to 8). It is the size of the buffer a caller builds an ImageBuffer into before it passes it by value to TextureGroup.uploadTexture. One recipe: the copy of the description in ImageBuffer.ctorFromDescription (0x5a4bb80 on 26.60; the anchor of that entry's recipe 0: `mov eax,[rsi+..]; mov rcx,[rbp+..]; mov [rcx+..],eax; movups xmm0,[rsi]; movups [rcx+..],xmm0; mov edx,[rsi]; mov r8d,[rsi+..]; mov eax,edx; or eax,r8d`), read as the displacement of `mov [rcx+..],eax` (the last member of the description, its depth, 28h), then `add 8` (4 bytes of depth, padded to 8). Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "pattern" {
      bytes  = "8B 46 ?? 48 8B 4D ?? 89 41 ?? 0F 10 06 0F 11 41 ?? 8B 16 44 8B 46 ?? 89 D0 44 09 C0"
      offset = 7
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
    step "add" { value = 8 }
  }

  check "range" {
    min = 32
    max = 96
  }
}

entry "ImageBuffer.mData" {
  kind     = "field"
  notes    = "The offset of the data pointer of the mce::Blob in cg::ImageBuffer: 8h on 26.60 (the blob deleter at +0, the byte size at +10h). Recipe 0 is the start of ImageBuffer.ctorFromDescription (0x5a4bb80 on 26.60), which stores the deleter and clears the data pointer and the size (`lea rax,[rip+..]; mov [rcx],rax; lea rax,[rcx+..]; mov [rbp+..],rax; xorps xmm0,xmm0; movups [rcx+..],xmm0; movups [rcx+..],xmm0`), read as the displacement of `lea rax,[rcx+..]`; the same inlined blob start is in 30 functions, so the pattern is scanned in the constructor only. Recipe 1 starts at ImageBuffer.ctorFromDescription, takes the call that allocates the bytes (`imul eax,r8d; mov edx,eax; call`, the blob resize, 0x3c7480 on 26.60) and reads, in that function, the load of the old data pointer and size (`mov rdx,[rdi+..]; mov r8,[rdi+..]; cmp rsi,r8`), the displacement of the first. They are in two functions. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "ref" { entry = "ImageBuffer.ctorFromDescription" }
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 41 ?? 48 89 45 ?? 0F 57 C0 0F 11 41 ?? 0F 11 41 ??"
      offset = 10
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "ImageBuffer.ctorFromDescription" }
    step "pattern" {
      bytes  = "41 0F AF C0 89 C2 E8"
      offset = 6
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
    step "pattern" {
      bytes  = "48 8B 57 ?? 4C 8B 47 ?? 4C 39 C6"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 32
  }
}

entry "ImageBuffer.mSize" {
  kind     = "field"
  notes    = "The offset of the byte size of the mce::Blob in cg::ImageBuffer: 10h on 26.60 (after the deleter +0 and the data pointer +8). Both recipes start at ImageBuffer.ctorFromDescription (0x5a4bb80 on 26.60) and take the call that allocates the bytes (`imul eax,r8d; mov edx,eax; call`, the blob resize, 0x3c7480 on 26.60). Recipe 0 reads, in that function, the first instruction (`cmp [rcx+..],rdx; je`: the size is already right), the displacement of the cmp. Recipe 1 reads the load of the old data pointer and size (`mov rdx,[rdi+..]; mov r8,[rdi+..]; cmp rsi,r8`), the displacement of the second. They read two instructions of one function. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipes were written on 26.60 only; 26.51 was not checked."

  recipe {
    step "ref" { entry = "ImageBuffer.ctorFromDescription" }
    step "pattern" {
      bytes  = "41 0F AF C0 89 C2 E8"
      offset = 6
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
    step "pattern" {
      bytes  = "48 39 51 ?? 0F 84"
      offset = 0
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "ref" { entry = "ImageBuffer.ctorFromDescription" }
    step "pattern" {
      bytes  = "41 0F AF C0 89 C2 E8"
      offset = 6
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
    step "pattern" {
      bytes  = "48 8B 57 ?? 4C 8B 47 ?? 4C 39 C6"
      offset = 4
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 8
    max = 40
  }
}

entry "ImageBuffer.mDeleter" {
  kind     = "field"
  notes    = "The offset of the deleter function pointer of the mce::Blob in cg::ImageBuffer: 0 on 26.60 (the data pointer at +8, the byte size at +10h). TextureGroup.uploadTexture calls it on the data pointer when it destroys the by-value buffer. One recipe: the start of ImageBuffer.ctorFromDescription (0x5a4bb80 on 26.60), which stores the deleter first (`lea rax,[rip+..]; mov [rcx],rax; lea rax,[rcx+..]; mov [rbp+..],rax; xorps xmm0,xmm0; movups [rcx+..],xmm0; movups [rcx+..],xmm0`), read as the displacement of `mov [rcx],rax` (an operand without a displacement byte reads as 0); the same inlined blob start is in 30 functions, so the pattern is scanned in the constructor only. Every displacement, immediate and branch byte is a wildcard. The range check keeps the value plausible. The recipe was written on 26.60 only; 26.51 was not checked."

  recipe {
    step "ref" { entry = "ImageBuffer.ctorFromDescription" }
    step "pattern" {
      bytes  = "48 8D 05 ?? ?? ?? ?? 48 89 01 48 8D 41 ?? 48 89 45 ?? 0F 57 C0 0F 11 41 ?? 0F 11 41 ??"
      offset = 7
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 32
  }
}
