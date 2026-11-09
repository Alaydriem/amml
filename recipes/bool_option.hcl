entry "BoolOption.set" {
  kind      = "function"
  msvc_name = "?set@BoolOption@@QEAAX_N0@Z"
  notes     = "void BoolOption::set(bool value, bool saveOptionChange): rcx = the BoolOption, dl = the value, r8b = the save flag; it returns void. It returns at once when a lock is set (`cmp qword ptr [mImpl+238h],0; je`), asks the can-change std::function at mImpl+180h, returns when the value does not change (`cmp [rsi+10h],dl; je`), passes the value through the coerce std::function at +18h (its impl pointer at +50h), stores it in mValue (+10h), calls the value-changed notify on mImpl (Option+8), and when the flag is set calls the request-save std::function at mImpl+298h. A direct write to mValue skips the notify, so a mod calls this function. It is not virtual. Found from the BoolOption constructor (it passes OptionType 0 to Option::Option and stores mValue and mDefaultValue at +10h and +11h) and BoolOption::load (vtable slot 4: the same `cmp [rsi+10h],al`, coerce and notify), next to this function. Recipe 0 is the body after the can-change call: `cmp [rsi+disp],dl; je; mov ebx,r8d; mov rcx,[rsi+disp]; test rcx,rcx; je; mov [rsp+..],dl; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rsp+..]; call [guard]`, then `function_start_of`. Recipe 1 is a call site in a small game setter (it ORs the negated value into a flag at this+20h, then sets bool option 2CEh, 2CDh on 26.51, 2D1h on the Preview): the anchor `mov eax,edx; xor al,imm; or [rcx+disp],al; mov rcx,[rcx+disp]; mov rax,[rcx]; mov rax,[rax+disp]; lea rdx,[rsp+..]; mov r8d,imm; call [guard]` finds the function, then a `pattern_any` in that function finds the inline lock check and the call: alternative 0 (26.51 to 26.63) `cmp qword ptr [rax+disp],imm; jne; mov edx,ebx; mov r8b,imm; call`, with three leading wildcards so that the call is at offset 18 in both; alternative 1 (the Preview, which puts a type assert before the check and reloads rcx) `cmp qword ptr [rax+disp],imm; jne; mov rcx,rsi; mov edx,ebx; mov r8b,imm; call`. It is read as a branch target. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard (no kept bytes). Each pattern is unique in every build and the two recipes agree. Check: function_start. Not hooked: a mod calls it. Values: 0x920420 in 26.51, 0x9c3120 in 26.60, 0x9c3130 in 26.62, 0x9c3060 in 26.63, 0x1960550 in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23). What changed: only the address up to 26.63; on the Preview the callers assert `mOptionType == 0` (Bool) before the lock check, and the body is the same. The auto-jump option (save tag ctrl_autojump, id 186h) is not a BoolOption: it is an InputModeBoolOption (OptionType 1); see InputModeBoolOption.set."

  recipe {
    step "pattern" {
      bytes = "38 56 ?? 74 ?? 44 89 C3 48 8B 4E ?? 48 85 C9 74 ?? 88 54 24 ?? 48 8B 01 48 8B 40 ?? 48 8D 54 24 ?? FF 15"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes = "89 D0 34 ?? 08 41 ?? 48 8B 49 ?? 48 8B 01 48 8B 80 ?? ?? ?? ?? 48 8D 54 24 ?? 41 B8 ?? ?? ?? ?? FF 15"
    }
    step "pattern_any" {
      patterns = [
        "?? ?? ?? 48 83 B8 ?? ?? ?? ?? ?? 75 ?? 89 DA 41 B0 ?? E8",
        "48 83 B8 ?? ?? ?? ?? ?? 75 ?? 48 89 F1 89 DA 41 B0 ?? E8",
      ]
      offset = 18
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}

entry "InputModeBoolOption.set" {
  kind      = "function"
  msvc_name = "?set@InputModeBoolOption@@QEAAXW4InputMode@@_N1@Z"
  notes     = "void InputModeBoolOption::set(InputMode mode, bool value, bool saveOptionChange): rcx = the InputModeBoolOption, edx = the InputMode (1, 2 or 3: the constructor fills only those keys), r8b = the value, r9b = the save flag; it returns void. It finds the mode in the std::unordered_map<InputMode,bool> mValues at +10h (FNV-1a hash of the four key bytes, list head at +18h, buckets at +28h, mask at +40h), returns when the stored bool (node+14h) is the value, else stores it, calls the per-mode value-changed notify at mImpl+80h with the mode, and when the flag is set calls the request-save std::function at mImpl+298h. A mode that is not a key falls through to the list head (the game has no such check); pass 1 (keyboard and mouse), 2 (touch) or 3 (gamepad). Found from the auto-jump option (save tag ctrl_autojump, id 186h): its constructor passes OptionType 1 to Option::Option and fills mValues (+10h) and mDefaultValues (+50h) for modes 1, 2 and 3. Recipe 0 is the body at the compare and store: `cmp [reg+disp],reg8; je; mov [reg+disp],reg8; mov [rsp+..],reg; mov rcx,[rsi+disp]; sub rcx,imm; lea r8,[rsp+..]; mov rdx,rsi; call`, then `function_start_of`; register bytes are wildcards because the Preview uses r14/bpl/ebx where 26.51 to 26.63 use rax/r8b/edx. Recipe 1 is the call site in the reset-to-default function (it switches on mOptionType at mImpl+1D4h; case 1 looks the mode up in mDefaultValues at +50h and calls this function with the default): `mov rax,[rsp+..]; movzx r8d,byte ptr [rax+disp]; mov rcx,rsi; mov edx,edi; mov r9d,ebx; call`, read as a branch target at offset 18. The case for InputModeFloatOption next to it loads the value with `movss`, so the pattern is unique without kept bytes. The two recipes are in two functions. Every immediate, displacement and branch byte is a wildcard. Each pattern is unique in every build and the two recipes agree. Check: function_start. Not hooked: a mod calls it. The MSVC name is built from the signature (the exe has no symbols for it). Values: 0x91d000 in 26.51, 0x9bfb20 in 26.60, 0x9bfb30 in 26.62, 0x9bfa60 in 26.63, 0x195ca00 in 6ae4eeee-4dd4fa7acacb4c41 (Preview 1.26.70.23). What changed on the Preview: a contract assert when the mode is missing, and other registers; the meaning is the same."

  recipe {
    step "pattern" {
      bytes = "?? 38 ?? ?? 74 ?? ?? 88 ?? ?? 89 ?? 24 ?? 48 8B 4E ?? 48 83 E9 ?? 4C 8D 44 24 ?? 48 89 F2 E8"
    }
    step "function_start_of" {}
  }

  recipe {
    step "pattern" {
      bytes  = "48 8B 44 24 ?? 44 0F B6 40 ?? 48 89 F1 89 FA 41 89 D9 E8"
      offset = 18
    }
    step "operand" {
      index = 0
      kind  = "branch_target"
    }
  }

  check "function_start" {}
}
