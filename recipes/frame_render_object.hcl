entry "FrameRenderObject.mViews" {
  kind     = "field"
  notes    = "Offset of the first-element pointer of FrameRenderObject::mViews (std::vector<ViewRenderObject, LinearAllocator<ViewRenderObject>>): 10h on 26.51. The vector has a 16-byte stateful allocator stored before the three pointers, so begin is at +10h and end at +18h. The FrameRenderObject deleter walks [frame+10h]..[frame+18h] in steps of 3300h (13056, the ViewRenderObject size in the header) and destroys each view: `mov rdi,[rdx]; test rdi,rdi; je; mov rsi,[rdi+10h]; test rsi,rsi; je; lea rbx,[rdi+10h]; mov r14,[rdi+18h]; cmp rsi,r14`. The recipe reads the first displacement. Range window 0..18h."

  recipe {
    step "pattern" {
      bytes  = "48 8B 3A 48 85 FF 0F 84 ?? ?? ?? ?? 48 8B 77 ?? 48 85 F6 74 ?? 48 8D 5F ?? 4C 8B 77 ?? 4C 39 F6"
      offset = 12
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 24
  }
}
