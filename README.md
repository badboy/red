# red - A Rust Editor

*red* is a line-oriented text editor, based on the popular [*ed* utility](http://pubs.opengroup.org/onlinepubs/009604599/utilities/ed.html).
It has two modes: *command mode* and *input mode*. In command mode the input characters are interpreted as commands, and in input mode they are interpreted as text.


Some of the history of *ed* is discussed in the article ["Where Vim Came From"](https://twobithistory.org/2018/08/05/where-vim-came-from.html).


## Installation

```
cargo install red-editor
```

## Usage

```
$ red file.txt
a
Hello World!
.
w
13
1,$p
Hello World!
q
```

## Available commands

* `(.,.)p` - Print the addressed lines to standard output.
* `(.,.)n` - Print the addressed lines to standard output, preceding each line by its line number and a &lt;tab&gt;.
* `(.,.)d` - Delete the addressed lines from the buffer.
* `(.,.)w [file]` - Write the addressed lines to the named file. The pathname is remembered for following writes.
* `(.)a` - Append text after the addressed line. End text input with a single `.` in a line.
* `(.)i` - Insert text before the addressed line. End text input with a single `.` in a line.
* `h` - Write a short message to standard output that explains the reason for the most recent `?`.
* `e [file]` - Delete the entire contents of the buffer and read the specified file into the buffer.
* `(.,.)c` - Delete the addressed line, then accept input text to replace these lines.
* `(.)r [file]` - Read contents of another file and insert into the buffer.
* `(.,.)maddress` - Move the addressed lines after the line addressed by `address`.

## Not (yet) implemented

* `/` - Addressing lines by search.
* `(.,.)g/RE/command` - Global regular expression search.
* `'x` - Marking lines with a name.

## License

MIT. See [LICENSE](LICENSE).

---
This file was written with *red*.
