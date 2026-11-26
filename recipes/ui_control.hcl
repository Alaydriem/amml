entry "UIControl.mCachedPosition" {
  kind     = "field"
  notes    = "Offset in UIControl of mCachedPosition, the glm::vec2 (two f32) absolute top-left of the control in GUI units: 10h on 26.60 and 26.62 (after the enable_shared_from_this weak pointer at +0). It is valid while the dirty bit (UIControl.mCachedPositionDirty) is clear; UIControl::_setCachedPosition (UIControl._setCachedPosition) rebuilds it as the parent's cached position plus mParentRelativePosition (+40h) and clears the bit. Recipe 0 is UIScene::getAreaOfControlByName (slot 52 of UIScene.vtable): its head up to the call of the VisualTree name lookup (`mov rsi,rdx; mov rax,[rcx+..]; pxor xmm0,xmm0; movdqu [rdx],xmm0; mov rcx,[rax+..]; lea rdx,[rbp+..]; xor r9d,r9d; call`), then, in the same function, the dirty test up to the _setCachedPosition call (`mov rdi,[rbp+..]; test rdi,rdi; je; test byte ptr [rdi+..],..; je; mov rcx,rdi; call`), then `movsd xmm0,[rdi+..]; movsd xmm1,[rdi+..]; movss xmm2,[rdi+..]` (position, size, position y), read as the displacement of the first load. Recipe 1 is the end of UIControl::_setCachedPosition (`movsd xmm0,[rsi+..]; addps xmm6,xmm0; movlps [rsi+..],xmm6; and byte ptr [rsi+..],..`), read as the displacement of the store. Each pattern step ends at a call or starts after one, so no pattern runs past a call (26.60 and 26.62 put a nop after these calls). The recipes are in two functions and share no pattern bytes. Every displacement, immediate and branch byte is a wildcard. The check is a window around the value. Value 10h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62). What changed in %s: the compiler now writes the zeroing of the out-rect as xorps and movups (not pxor and movdqu), the visited-control test as a near je, and the three movsd loads of the area rectangle in other registers, so the head, the dirty-test and the load patterns of the area function each got a second alternative (pattern_any, one alternative per shape; each matches once on its own build and zero times on the other). The second area function (the std::function body that walks from the root control) has a new head too. The area function is slot 52 of the UIScene vtable on both builds. Class: would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" { patterns = ["48 89 D6 48 8B 41 ?? 66 0F EF C0 F3 0F 7F 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8", "48 89 D6 48 8B 41 ?? 0F 57 C0 0F 11 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8"] }
    step "pattern_any" { patterns = ["48 8B 7D ?? 48 85 FF 74 ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??", "48 8B 7D ?? 48 85 FF 0F 84 ?? ?? ?? ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??"] }
    step "pattern_any" { patterns = ["F2 0F 10 47 ?? F2 0F 10 4F ?? F3 0F 10 57 ??", "F2 0F 10 47 ?? F2 0F 10 7F ?? 0F 28 F0", "F2 0F 10 47 ?? F2 0F 10 4F ?? 0F 28 D0"] }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "F2 0F 10 46 ?? 0F 58 F0 0F 13 76 ?? 80 66 ?? ??"
      offset = 8
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 0
    max = 64
  }
}

entry "UIControl.mCachedPositionDirty" {
  kind     = "field"
  notes    = "Offset in UIControl of the byte that holds the mCachedPositionDirty bit (bit 0, mask 1): 18h on 26.60 and 26.62. The same byte holds mHover (bit 1), mVisible (bits 2-3) and mIsVisibleInTree (bit 4, UIControl.mIsVisibleInTree). When the bit is set, mCachedPosition is stale and UIControl::_setCachedPosition must run first, as UIScene::getAreaOfControlByName does. Recipe 0 is UIScene::getAreaOfControlByName: its head up to the name lookup call (`mov rsi,rdx; ...; xor r9d,r9d; call`), then, in the same function, `mov rdi,[rbp+..]; test rdi,rdi; je; test byte ptr [rdi+..],..; je; mov rcx,rdi; call <_setCachedPosition>`, read as the displacement of the test. Recipe 1 is the end of UIControl::_setCachedPosition (`movsd xmm0,[rsi+..]; addps xmm6,xmm0; movlps [rsi+..],xmm6; and byte ptr [rsi+..],..`), read as the displacement of the `and` that clears the bit. The bit masks (1 in the test, 0FEh in the and) are wildcarded like every other immediate; the mask itself is the entry UIControl.mCachedPositionDirtyMask. The recipes are in two functions and share no pattern bytes. No pattern step runs past a call: the head ends at the lookup call and the next step starts after it (26.60 and 26.62 put a nop after these calls). Every displacement, immediate and branch byte is a wildcard. The check is a window around the value. Value 18h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62). What changed in %s: the compiler now writes the zeroing of the out-rect as xorps and movups (not pxor and movdqu), the visited-control test as a near je, and the three movsd loads of the area rectangle in other registers, so the head, the dirty-test and the load patterns of the area function each got a second alternative (pattern_any, one alternative per shape; each matches once on its own build and zero times on the other). The second area function (the std::function body that walks from the root control) has a new head too. The area function is slot 52 of the UIScene vtable on both builds. Class: would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" { patterns = ["48 89 D6 48 8B 41 ?? 66 0F EF C0 F3 0F 7F 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8", "48 89 D6 48 8B 41 ?? 0F 57 C0 0F 11 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8"] }
    step "pattern_any" { patterns = ["48 8B 7D ?? 48 85 FF 74 ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??", "48 8B 7D ?? 48 85 FF 0F 84 ?? ?? ?? ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??"] }
    step "pattern" { bytes = "F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??" }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern" {
      bytes  = "F2 0F 10 46 ?? 0F 58 F0 0F 13 76 ?? 80 66 ?? ??"
      offset = 12
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 64
  }
}

entry "UIControl.mIsVisibleInTree" {
  kind     = "field"
  notes    = "Offset in UIControl of the byte that holds the mIsVisibleInTree bit (bit 4, mask 10h): 18h on 26.60 and 26.62, the same byte as UIControl.mCachedPositionDirty. The bit is set when the control and all its ancestors are visible (a control hidden by a #visible binding, such as crafting.player_inventory in the creative layout, has it clear). Recipe 0 is the name predicate of VisualTree's control-by-name lookup (the call operator of the std::function the lookup builds): `mov rax,[rcx+..]; cmp byte ptr [rax],..; jne; test byte ptr [rdx+..],..; je; mov rcx,[rcx+..]; cmp qword ptr [rcx+..],..; mov rax,rcx; jb; mov rax,[rcx]; mov r8,[rcx+..]; mov rcx,[rdx+..]; cmp qword ptr [rdx+..],..; jb; mov rdx,[rdx+..]` (when the visible-only flag is set, a control without the bit does not match), read as the displacement of the test. The mask 10h is wildcarded like every other immediate; the mask itself is the entry UIControl.mIsVisibleInTreeMask. One recipe and a window check, as for a member offset. Value 18h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62). What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the name predicate of the control lookup loads [rcx+10h] before the compare, so the test is 4 bytes later; recipe 0 is now a pattern_any for the whole predicate (one alternative per shape) followed by the common test. Class: would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" { patterns = ["48 8B 41 ?? 80 38 ?? 75 ?? F6 42 ?? ?? 74 ?? 48 8B 49 ?? 48 83 79 ?? ?? 48 89 C8 72 ?? 48 8B 01 4C 8B 41 ?? 48 8B 4A ?? 48 83 7A ?? ?? 72 ?? 48 8B 52 ??", "48 8B 41 ?? 48 8B 49 ?? 80 38 ?? 75 ?? F6 42 ?? ?? 74 ?? 48 83 79 ?? ?? 48 89 C8 72 ?? 48 8B 01 4C 8B 41 ?? 48 8B 4A ?? 48 83 7A ?? ?? 72 ?? 48 8B 52 ??"] }
    step "pattern" {
      bytes  = "F6 42 ?? ?? 74 ??"
      offset = 0
    }
    step "operand" {
      index = 0
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 64
  }
}

entry "UIControl.mName" {
  kind     = "field"
  notes    = "Offset in UIControl of mName, the std::string name of the control (the JSON UI element name, for example root_panel): 20h on 26.60 and 26.62. Recipe 0 is UIScene::getRawScreenName (slot 58 of UIScene.vtable), which copies the name of the visual tree's root control: `mov rsi,rdx; mov rax,[rcx+..]; mov rax,[rax+..]; test rax,rax; je; mov rcx,[rax+..]; ...; mov r15,[rax+..]; test r15,r15; je; cmp qword ptr [r15+..],..; jb; mov rdi,[r15+..]`, read as the displacement of the load of the heap buffer pointer (the string starts there). Recipe 1 is the name predicate of VisualTree's control-by-name lookup: `... mov rcx,[rdx+..]; cmp qword ptr [rdx+..],..; jb; mov rdx,[rdx+..]`, read as the displacement of the last load (the same string of the control in rdx). The recipes are in two functions; recipe 1 shares no bytes with recipe 0. Every displacement, immediate and branch byte is a wildcard. The check is a window around the value. Value 20h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62). What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): recipe 1 (the predicate) is a pattern_any as for mIsVisibleInTree. Recipe 0: the root-name getter no longer copies the name in its own body; it calls a helper that returns the root control and reads the name there (cmp qword ptr [rax+38h],10h; jb; mov rax,[rax+20h]), so recipe 0 has a second alternative for that helper, written with leading wildcards so that the load is at the same offset. Class: would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" {
      patterns = [
        "48 89 D6 48 8B 41 ?? 48 8B 40 ?? 48 85 C0 0F 84 ?? ?? ?? ?? 48 8B 48 ?? 48 85 C9 74 ?? F0 FF 41 ?? 4C 8B 70 ?? EB ?? 45 31 F6 4C 8B 78 ?? 4D 85 FF 74 ?? 49 83 7F ?? ?? 72 ?? 49 8B 7F ??",
        "?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? 48 8B 45 ?? 48 8B 7D ?? 48 89 45 ?? 48 89 7D ?? 48 85 C0 74 ?? 48 8B 48 ?? 48 83 78 ?? ?? 0F 82 ?? ?? ?? ?? 48 8B 40 ??",
      ]
      offset = 58
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" { patterns = ["48 8B 41 ?? 80 38 ?? 75 ?? F6 42 ?? ?? 74 ?? 48 8B 49 ?? 48 83 79 ?? ?? 48 89 C8 72 ?? 48 8B 01 4C 8B 41 ?? 48 8B 4A ?? 48 83 7A ?? ?? 72 ?? 48 8B 52 ??", "48 8B 41 ?? 48 8B 49 ?? 80 38 ?? 75 ?? F6 42 ?? ?? 74 ?? 48 83 79 ?? ?? 48 89 C8 72 ?? 48 8B 01 4C 8B 41 ?? 48 8B 4A ?? 48 83 7A ?? ?? 72 ?? 48 8B 52 ??"] }
    step "pattern" {
      bytes  = "48 83 7A ?? ?? 72 ?? 48 8B 52 ??"
      offset = 7
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 16
    max = 96
  }
}

entry "UIControl.mSize" {
  kind     = "field"
  notes    = "Offset in UIControl of mSize, the glm::vec2 (two f32) width and height of the control in GUI units: 48h on 26.60 and 26.62 (mParentRelativePosition at +40h before it). Recipe 0 is UIScene::getAreaOfControlByName: its head up to the name lookup call, then, in the same function, the dirty test up to the _setCachedPosition call, then `movsd xmm0,[rdi+..]; movsd xmm1,[rdi+..]; movss xmm2,[rdi+..]`, read as the displacement of the second load (the RectangleArea is position and position + size). Recipe 1 is a second area function that walks the visual tree from the root control (`mov rsi,rdx; mov rax,[rcx+..]; mov ecx,[r9]; mov rax,[rax+..]; mov [rbp+..],ecx; pxor xmm0,xmm0; movdqa [rbp+..],xmm0`), then the same three loads in that function, read as the displacement of the second load. The two recipes are in two functions; the second patterns of both recipes are the same bytes, so the independence is the different first anchors. No pattern step runs past a call: the head ends at the lookup call and the next step starts after it (26.60 and 26.62 put a nop after these calls). Every displacement, immediate and branch byte is a wildcard. The check is a window around the value. Value 48h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62). What changed in %s: the compiler now writes the zeroing of the out-rect as xorps and movups (not pxor and movdqu), the visited-control test as a near je, and the three movsd loads of the area rectangle in other registers, so the head, the dirty-test and the load patterns of the area function each got a second alternative (pattern_any, one alternative per shape; each matches once on its own build and zero times on the other). The second area function (the std::function body that walks from the root control) has a new head too. The area function is slot 52 of the UIScene vtable on both builds. Class: would have survived with a better recipe. Recipe 1 is a second area function reached through a std::function vtable; on 6afa4e47 its head uses xorps and movaps, found again by the same vtable shape (two thunks, then the function in slot 4). No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" { patterns = ["48 89 D6 48 8B 41 ?? 66 0F EF C0 F3 0F 7F 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8", "48 89 D6 48 8B 41 ?? 0F 57 C0 0F 11 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8"] }
    step "pattern_any" { patterns = ["48 8B 7D ?? 48 85 FF 74 ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??", "48 8B 7D ?? 48 85 FF 0F 84 ?? ?? ?? ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??"] }
    step "pattern_any" {
      patterns = ["F2 0F 10 47 ?? F2 0F 10 4F ?? F3 0F 10 57 ??", "F2 0F 10 47 ?? F2 0F 10 7F ?? 0F 28 F0", "F2 0F 10 47 ?? F2 0F 10 4F ?? 0F 28 D0"]
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  recipe {
    step "pattern_any" { patterns = ["48 89 D6 48 8B 41 ?? 41 8B 09 48 8B 40 ?? 89 4D ?? 66 0F EF C0 66 0F 7F 45 ??", "48 89 D6 48 8B 41 ?? 41 8B 11 48 8B 48 ?? 89 55 ?? 0F 57 C0 0F 29 45 ??"] }
    step "pattern_any" {
      patterns = ["F2 0F 10 47 ?? F2 0F 10 4F ?? F3 0F 10 57 ??", "F2 0F 10 47 ?? F2 0F 10 7F ?? 0F 28 F0", "F2 0F 10 47 ?? F2 0F 10 4F ?? 0F 28 D0"]
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 32
    max = 128
  }
}

entry "UIControl.mChildren" {
  kind     = "field"
  notes    = "Offset in UIControl of mChildren, the std::vector<std::shared_ptr<UIControl>> of child controls (begin, end, capacity; 10h-byte elements, the control pointer first): 98h on 26.60 and 26.62. Recipe 0 is the recursive collector of VisualTree's control-by-name lookup: after the push of a matching control (`add qword ptr [rsi+..],..`), `mov rbx,[rdi+..]; mov r14,[rdi+..]; lea r15,[rbp+..]; cmp rbx,r14; je`, read as the displacement of the begin load (the loop then steps rbx by 10h and recurses on [rbx]). One recipe and a window check, as for a member offset; the end load at +8 from it is in the same pattern. Every displacement, immediate and branch byte is a wildcard. Value 98h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62). What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the recursive collector keeps the begin and end loads of mChildren but with r12 and r13 (not rbx and r14), and no `lea r15` follows; a second alternative matches the push followed by the two loads. The collector is the callee that the name lookup reaches (the lookup, a wrapper, then the collector). Evidence: the push of the matching control (`add qword ptr [rsi+8],10h`) followed by the loads at 98h and 0A0h, the element stride 10h and the self call in the loop on both builds. Class: would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" {
      patterns = ["48 83 46 ?? ?? 48 8B 9F ?? ?? ?? ?? 4C 8B B7 ?? ?? ?? ?? 4C 8D 7D ?? 4C 39 F3 74 ??", "48 83 46 ?? ?? 4C 8B A7 ?? ?? ?? ?? 4C 8B AF ?? ?? ?? ??"]
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
  }

  check "range" {
    min = 96
    max = 224
  }
}

entry "UIControl._setCachedPosition" {
  kind     = "function"
  notes    = "void UIControl::_setCachedPosition() const: rcx the UIControl; no other arguments, returns nothing. It locks the parent weak pointer (+88h), takes the parent's cached position (calling itself first when the parent's dirty bit is set), adds mParentRelativePosition (+40h), stores the sum in mCachedPosition (UIControl.mCachedPosition) and clears the dirty bit (UIControl.mCachedPositionDirty). Recipe 0 is the call site in UIScene::getAreaOfControlByName: its head up to the name lookup call (`mov rsi,rdx; ...; xor r9d,r9d; call`), then, in the same function, `mov rdi,[rbp+..]; test rdi,rdi; je; test byte ptr [rdi+..],..; je; mov rcx,rdi; call <this>`, read as the call target. Recipe 1 is the end of the body (`movsd xmm0,[rsi+..]; addps xmm6,xmm0; movlps [rsi+..],xmm6; and byte ptr [rsi+..],..`), then function_start_of. The two recipes are in two functions and share no pattern bytes. No pattern step runs past a call: the head ends at the lookup call and the next step starts after it (26.60 and 26.62 put a nop after these calls). Every displacement, immediate and branch byte is a wildcard. The check is function_start (recipe 0 does not end with function_start_of, so it counts). Value 1C68360h on 6ad79ee4-17d51891a1119914 (26.60) and 1C68220h on 6aea437e-7eff13daabb430da (26.62). What changed in %s: the compiler now writes the zeroing of the out-rect as xorps and movups (not pxor and movdqu), the visited-control test as a near je, and the three movsd loads of the area rectangle in other registers, so the head, the dirty-test and the load patterns of the area function each got a second alternative (pattern_any, one alternative per shape; each matches once on its own build and zero times on the other). The second area function (the std::function body that walks from the root control) has a new head too. The area function is slot 52 of the UIScene vtable on both builds. Class: would have survived with a better recipe. Evidence for the new build: the end-of-body pattern of the old function matches once, and the function has 172 direct callers (173 on 6aed1906), including its own recursive call and the call in the area function. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" { patterns = ["48 89 D6 48 8B 41 ?? 66 0F EF C0 F3 0F 7F 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8", "48 89 D6 48 8B 41 ?? 0F 57 C0 0F 11 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8"] }
    step "pattern_any" { patterns = ["48 8B 7D ?? 48 85 FF 74 ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??", "48 8B 7D ?? 48 85 FF 0F 84 ?? ?? ?? ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??"] }
    step "pattern" {
      bytes  = "F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??"
      offset = 9
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  recipe {
    step "pattern" { bytes = "F2 0F 10 46 ?? 0F 58 F0 0F 13 76 ?? 80 66 ?? ??" }
    step "function_start_of" {}
  }

  check "function_start" {}
}

entry "UIControl.mCachedPositionDirtyMask" {
  kind     = "field"
  notes    = "The bit mask of mCachedPositionDirty in the byte at UIControl.mCachedPositionDirty: 1 (bit 0) on 26.60 and 26.62. A mod tests byte & mask. Recipe 0 is UIScene::getAreaOfControlByName: its head up to the name lookup call (`mov rsi,rdx; ...; xor r9d,r9d; call`), then, in the same function, `mov rdi,[rbp+..]; test rdi,rdi; je; test byte ptr [rdi+..],..; je; mov rcx,rdi; call <_setCachedPosition>`, read as the immediate of the test. Recipe 1 is the same test in UIControl::_setCachedPosition itself, which checks the parent's dirty bit before it calls itself: ref UIControl._setCachedPosition, then, in that function, `test rdi,rdi; je; test byte ptr [rdi+..],..; je; mov rcx,rdi; call`, read as the immediate of the test. No pattern step runs past a call. The recipes are in two functions; the second pattern of recipe 0 and the pattern of recipe 1 share their shape, so the independence is the different first anchors. Every displacement, immediate and branch byte is a wildcard. The check is a window over the bit masks of one byte. Value 1 on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62). What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): recipe 0 reads the test in the area function, which now has a near je before the test, so its anchor is a pattern_any followed by the shared tail of the test. Class: would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" { patterns = ["48 89 D6 48 8B 41 ?? 66 0F EF C0 F3 0F 7F 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8", "48 89 D6 48 8B 41 ?? 0F 57 C0 0F 11 02 48 8B 48 ?? 48 8D 55 ?? 45 31 C9 E8"] }
    step "pattern_any" { patterns = ["48 8B 7D ?? 48 85 FF 74 ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??", "48 8B 7D ?? 48 85 FF 0F 84 ?? ?? ?? ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??"] }
    step "pattern" { bytes = "F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??" }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  recipe {
    step "ref" { entry = "UIControl._setCachedPosition" }
    step "pattern" {
      bytes  = "48 85 FF 74 ?? F6 47 ?? ?? 74 ?? 48 89 F9 E8 ?? ?? ?? ??"
      offset = 5
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 1
    max = 128
  }
}

entry "UIControl.mIsVisibleInTreeMask" {
  kind     = "field"
  notes    = "The bit mask of mIsVisibleInTree in the byte at UIControl.mIsVisibleInTree: 10h (bit 4) on 26.60 and 26.62. A mod tests byte & mask. Recipe 0 is the name predicate of VisualTree's control-by-name lookup (`mov rax,[rcx+..]; cmp byte ptr [rax],..; jne; test byte ptr [rdx+..],..; je; mov rcx,[rcx+..]; ...; mov rdx,[rdx+..]`), read as the immediate of the test (the visible-only filter). One recipe and a window check, the same shape as UIControl.mIsVisibleInTree. Every displacement, immediate and branch byte is a wildcard. Value 10h on 6ad79ee4-17d51891a1119914 (26.60) and 6aea437e-7eff13daabb430da (26.62). What changed in 6afa4e47-dadc375797a3d31a (1.26.70.26): the name predicate of the control lookup loads [rcx+10h] before the compare, so the test is 4 bytes later; recipe 0 is now a pattern_any for the whole predicate (one alternative per shape) followed by the common test. Class: would have survived with a better recipe. No build gate: builds before 26.63 are not supported."

  recipe {
    step "pattern_any" { patterns = ["48 8B 41 ?? 80 38 ?? 75 ?? F6 42 ?? ?? 74 ?? 48 8B 49 ?? 48 83 79 ?? ?? 48 89 C8 72 ?? 48 8B 01 4C 8B 41 ?? 48 8B 4A ?? 48 83 7A ?? ?? 72 ?? 48 8B 52 ??", "48 8B 41 ?? 48 8B 49 ?? 80 38 ?? 75 ?? F6 42 ?? ?? 74 ?? 48 83 79 ?? ?? 48 89 C8 72 ?? 48 8B 01 4C 8B 41 ?? 48 8B 4A ?? 48 83 7A ?? ?? 72 ?? 48 8B 52 ??"] }
    step "pattern" {
      bytes  = "F6 42 ?? ?? 74 ??"
      offset = 0
    }
    step "operand" {
      index = 1
      kind  = "immediate"
    }
  }

  check "range" {
    min = 1
    max = 128
  }
}
