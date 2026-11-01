entry "ViewRenderObject.mViewData" {
  kind     = "field"
  notes    = "Offset of `mViewData` (ViewRenderData, inline, 140 bytes) in ViewRenderObject: 0 in 26.51 (LeviLamina header: the first member; the TypedStorage sum of all members is 0x3300, the stride of the view-object vector in the render function that calls LevelRendererPlayer::createViewRenderObject). The code reads `viewObject.mViewData.mCameraPos` as one displacement from the ViewRenderObject pointer, so the value read is the sum mViewData + ViewRenderData::mCameraPos; both are offsets, so a sum of 0 means each is 0. The range check 0..0 makes the entry refuse instead of guess if the sum ever moves (it could then not be split). Recipe 0 is the name-tag renderer that LevelRendererPlayer::callRenderNameTags (slot 11 of the LevelRendererPlayer vtable) calls once per name tag with the view object in rdx: it keeps the view object in r14 and computes camera position minus tag position (`movss xmm1,[r14+8]; subss xmm1,[rdi+..]; movss xmm2,[r14]; movss xmm0,[r14+4]; subss xmm0,[rdi+..]; subss xmm2,[rdi+..]`); it later subtracts [r14+0Ch..14h] (mCameraTargetPos). The recipe reads the displacement of the camera x load (operand 1). Recipe 1 is the weather renderer that LevelRendererCamera::render calls with the view object in rdx and its weather state (view object + 830h) in r8; it saves the view object at [rbp+..], tests mIsFancyRendering at +83h, and reloads it to read the camera position (`mov rcx,[rbp+..]; movss xmm1,[rcx+8]; cvttps2dq; cvtdq2ps; subss; movss xmm0,[rcx]; movss xmm2,[rcx+4]`). The recipe reads the displacement of `movss xmm0,[rcx]` (operand 1). Neither load has a displacement byte, so the operand is 0. The two functions are independent and share no pattern bytes. Every other displacement byte is a wildcard. Recipe 0 is a pattern_any with one alternative per build shape, each starting at the x load of the camera position: 26.51 `movss xmm2,[r14]; movss xmm0,[r14+4]; subss; subss` (after the z load), 26.60 `movss xmm0,[r14]; subss xmm0,[rbx+50h]; xorps; ucomiss; ...` (the x difference is tested against zero first). Both keep the `[r14]` ModRM byte with no displacement, which is the fact the entry reads. What changed in 6ad79ee4-17d51891a1119914: the name-tag renderer was rewritten around the x difference and the view object shrank by 20h (3300h to 32E0h; the name-tag vector that callRenderNameTags walks moved from +32E8h to +32C8h); the camera position stayed first, so the old pattern matched nothing but the value is the same. Class: real code change (code shape). Evidence for 0 on 26.60: (1) callRenderNameTags, found on 26.60 by a pattern drafted from the 26.51 function (the name-tag vector loop, unique on both builds), calls the 26.60 name-tag renderer with the view object in rdx; that renderer keeps it in r14, reads the camera x at [r14] and later subtracts [r14+0Ch..14h] (mCameraTargetPos); (2) recipe 1 (the weather renderer), drafted from 26.51, matches once on 26.60 and reads the camera x at [rcx] after the z at [rcx+8]."

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
