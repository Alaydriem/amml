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
