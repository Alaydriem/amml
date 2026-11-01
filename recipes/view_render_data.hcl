entry "ViewRenderData.mCameraPos" {
  kind     = "field"
  notes    = "Offset of `mCameraPos` (glm::vec3) in ViewRenderData: 0 in 26.51 (LeviLamina header: the first member, then mCameraTargetPos at 0Ch). The code reads it only through a ViewRenderObject, as one displacement that is the sum ViewRenderObject::mViewData + mCameraPos; both are offsets, so a sum of 0 means each is 0, and the range check 0..0 makes the entry refuse if the sum moves. Recipes 0 and 1 are the two loads described in ViewRenderObject.mViewData: the name-tag renderer (camera position minus tag position, `movss xmm2,[r14]`) and the weather renderer (`movss xmm0,[rcx]` after the z load at +8). Both functions also read the next vec3 at +0Ch..14h (mCameraTargetPos), which places the camera position in the first 12 bytes. Neither load has a displacement byte, so the operand is 0. Every other displacement byte is a wildcard. Recipe 0 is a pattern_any with one alternative per build shape, each starting at the x load of the camera position: 26.51 `movss xmm2,[r14]; movss xmm0,[r14+4]; subss; subss` (after the z load), 26.60 `movss xmm0,[r14]; subss xmm0,[rbx+50h]; xorps; ucomiss; ...` (the x difference is tested against zero first). Both keep the `[r14]` ModRM byte with no displacement, which is the fact the entry reads. What changed in 6ad79ee4-17d51891a1119914: the name-tag renderer was rewritten around the x difference and the view object shrank by 20h (3300h to 32E0h; the name-tag vector that callRenderNameTags walks moved from +32E8h to +32C8h); the camera position stayed first, so the old pattern matched nothing but the value is the same. Class: real code change (code shape). Evidence for 0 on 26.60: (1) callRenderNameTags, found on 26.60 by a pattern drafted from the 26.51 function (the name-tag vector loop, unique on both builds), calls the 26.60 name-tag renderer with the view object in rdx; that renderer keeps it in r14, reads the camera x at [r14] and later subtracts [r14+0Ch..14h] (mCameraTargetPos); (2) recipe 1 (the weather renderer), drafted from 26.51, matches once on 26.60 and reads the camera x at [rcx] after the z at [rcx+8]."

  recipe {
    step "pattern_any" {
      patterns = [
        "F3 41 0F 10 16 F3 41 0F 10 46 ?? F3 0F 5C 47 ?? F3 0F 5C 57 ??",
        "F3 41 0F 10 06 F3 0F 5C 43 ?? 45 0F 57 ?? 41 0F 2E ?? F3 0F 10 2D ?? ?? ?? ?? 0F 28 E0 75 ??",
      ]
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 8D ?? ?? ?? ?? F3 0F 10 49 ?? F3 0F 5B C0 0F 5B C0 F3 0F 5C C8 F3 0F 10 01 F3 0F 10 51 ?? F3 0F 5B DF"
      offset = 23
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 0
  }
}
