entry "NameTagRenderObject.size" {
  kind     = "field"
  notes    = "sizeof(NameTagRenderObject): 0x90 (144) in 26.51 (header sum: string 32, two material pointers 16, two colours 32, vec3 12, scale 4, variant 24, optional<Quaternion> 20, line height 4, aligned to 8). The value is the stride of the vector of tags, and the size check of the NameTagRenderObject mirror of the mod. Recipe 0 is the render loop over a view object name tags (vtable slot 11 of LevelRendererPlayer): `add r15,90h; cmp r15,r12; jne`, read as the immediate of the add. Recipe 1 is the element move loop of the vector insert function (0x4703c70): `add r8,90h; add rcx,90h; je`, read as the immediate of the first add. They are in two functions. Every register, displacement and branch byte is a wildcard; the immediate is the value itself. What changed in 6ad79ee4-17d51891a1119914 (26.60): the value is still 0x90. The element copy loop of the vector insert function uses other registers and ends with `cmp rdi,r12; jne` (`call element-copy; mov rcx,[rbp-28h]; add rcx,90h; add rdi,90h; cmp rdi,r12; jne`), so recipe 1 is a pattern_any with one alternative per build. Class: would have survived with a better recipe. Cold run result: degraded (recipe 1 no match, recipe 0 agreed)."

  recipe {
    step "pattern" {
      bytes  = "49 81 C7 ?? ?? ?? ?? 4D 39 E7 75 ?? 48 83 C4 ?? 5B 5F 5E 41 5C 41 5E 41 5F C3"
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "pattern_any" {
      patterns = [
        "49 81 C0 ?? ?? ?? ?? 48 81 C1 ?? ?? ?? ?? 0F 84",
        "48 81 C1 ?? ?? ?? ?? 48 81 C7 ?? ?? ?? ?? 4C 39 E7 75",
      ]
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 64
    max = 256
  }

}
