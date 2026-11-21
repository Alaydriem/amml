entry "InputHandler.mAnyInputHandler" {
  kind  = "field"
  notes = "Offset of `mAnyInputHandler` (an MSVC std::function<void(IClientInstance&)>, 40h bytes; its callable `_Impl` pointer is the last word, at +38h) in InputHandler: C0h in 26.63 (6aed1906-b127418e74d2f2f9) and in 70.26 (6afa4e47-dadc375797a3d31a). The game runs it for each input event; Client::note_activity calls it once (`_Do_call`, slot 2 of the callable, with the client), which resets the game's idle timer without an input event. How it was found: InputHandler::registerAnyInputHandler, called from MinecraftInputHandler::_registerInputHandlers on [this+20h] (MinecraftInputHandler.mInputHandler) with the lambda that publishes to the MinecraftInputHandler any-input subscribers (lambda 10 of _registerInputHandlers by its RTTI on 26.63; the same body on 70.26), moves its argument into [this+0C0h]; four sibling register functions of the same shape follow it and use 100h, 140h, 180h and 1C0h. The input dispatcher of InputHandler calls the handler: `mov rax,[rbp+..]; mov rcx,[rax+0F8h]; test rcx,rcx; je; mov rax,[rcx]; mov rax,[rax+10h]; mov rdx,[rbp+..]; call [guard]` with the client in rdx. Recipe 0 starts in the dispatcher at `lea rdx,[rip+..]; mov rcx,[rbp+..]; mov r8d,eax; call` (unique on both builds), then finds that call of the callable in the same function (the only one there; the same shape reads other members in other functions), reads the displacement of the callable load (0F8h) and subtracts 38h, the offset of the `_Impl` pointer in the MSVC std::function, to give the member. Both patterns end at a call. Every displacement, immediate and branch byte is a wildcard. Second evidence on both builds (not a recipe: the five register functions differ only in the member): the `add rsi,0C0h` of registerAnyInputHandler. The range check is a window around the value. One recipe and a range check, as for other members."

  recipe {
    step "pattern" {
      bytes = "48 8D 15 ?? ?? ?? ?? 48 8B 8D ?? ?? ?? ?? 41 89 C0 E8"
    }
    step "pattern" {
      bytes = "48 8B 88 ?? ?? ?? ?? 48 85 C9 0F 84 ?? ?? ?? ?? 48 8B 01 48 8B 40 ?? 48 8B 95 ?? ?? ?? ?? FF 15"
    }
    step "operand" {
      index = 1
      kind  = "displacement"
    }
    step "add" {
      value = -56
    }
  }

  check "range" {
    min = 128
    max = 384
  }
}
