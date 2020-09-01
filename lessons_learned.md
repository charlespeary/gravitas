# Things we don't have in C
- For now I see that most of the generic stuff is limited to `void*`, but it doesn't help much with the code duplication.
- `void*` is a special kind of pointer that takes everything - I don't know much about it yet, but it sounds like a great way to kill your app somewhere along the way.
- I feel like I did at least 3 mistakes that would potentially segfault somewhere along the way, but the compiler didn't even mention it.
- Before reaching the second chapter of Crafting Interpreters I already spent some time debugging with debugger and Valgrind. My first memory leak - yay!