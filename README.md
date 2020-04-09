# Collect memories

---

Collect memories is a tool to simplify a retrieval of media files lost in
unorganized chaos.

Tool finds files by using user specified file extensions,
lists them and provides functionality for user to further
filter found files and to copy them to specified output directory by
keeping directory structure intact as to not to create greater chaos.

User after copying files can use their file browser to further organize files.

**The main feature of this tool: before copying all matched files you can remove
all unneeded ones in an easy way!**

---

### User friendly

No need to be afraid!

Mouse is supported and directory picking choices are
done through operating system native graphical user interface window!

Everywhere else user can move around by using mouse, arrow keys,
<Tab>, <Shift> + <Tab> and <Enter>.

### Example Screenshots

![Extension and source directory interface](https://raw.github.com/PPakalns/Collect-Memories/master/doc/select.png)

Program first asks for list of file extensions (default list already provided)
and source directory from which to start file search.

![Found file list](https://raw.github.com/PPakalns/Collect-Memories/master/doc/list.png)

Then after scanning is done it outputs list of found files in
tree like view.

Often in the middle of needed files are some unneeded system or software
files.

**Unneeded files can be removed from the list by selecting directory or
specific file and pressing "Remove selected subtree ..." button or just by
pressing "r" key.**

![Found file list with removed system files](https://raw.github.com/PPakalns/Collect-Memories/master/doc/list_filtered.png)

Then the selected system directory with all items in it will be removed
from the list. See picture above where system files were unlisted.

![Memories copied successfully](https://raw.github.com/PPakalns/Collect-Memories/master/doc/copied.png)

After copying files output directory will contains found files with
original directory structure intact!
```
.
└── 2020
    ├── August
    │   ├── 2020-09-12.png
    │   └── 2020-09-16.png
    └── July
        ├── 2020-07-19.png
        └── 2020-07-3.png

3 directories, 4 files
```
Happy further memory organization!

---

### License

Copyright 2020 Pēteris Pakalns <peterispakalns@gmail.com>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
