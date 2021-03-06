![App preview image](app/tenXten.jpg)

# 10x10 game solver

This rust program solves the game described [here](https://chycho.blogspot.com/2014/01/an-exercise-for-mind-10-by-10-math.html).

It also contains a WebApp for playing (and solving) the game.
You can find the WebApp [here](https://keksboter.github.io/tenXten/). 

## Usage

```
$ tenxten --help
tenXten 0.1.0
A cli for solving the 10x10 number game

USAGE:
    tenxten [FLAGS] [OPTIONS] <column> <row>

FLAGS:
    -f, --find-all        find all solutions (takes very long)
    -h, --help            Prints help information
    -n, --no-animation    do not animate the solution
    -V, --version         Prints version information
    -v, --verbose         shows additional information

OPTIONS:
    -a, --animation-delay <animation-delay>    delay (ms) between frames in the animation [default: 50]
    -b, --board-size <board-size>              size (width and height) of the board [default: 10]

ARGS:
    <column>    start column
    <row>       start row
```

## Example

```
$ tenxten --board-size 10 --no-animation --verbose 3 3
Initial board:
╔═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╗
║▒▒▒│   │   │   │▒▒▒│   │   │   │   │   ║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║   │   │   │   │   │   │   │   │   │   ║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║   │   │  1│   │   │▒▒▒│   │   │   │   ║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║   │   │   │   │   │   │   │   │   │   ║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║▒▒▒│   │   │   │▒▒▒│   │   │   │   │   ║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║   │   │▒▒▒│   │   │   │   │   │   │   ║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║   │   │   │   │   │   │   │   │   │   ║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║   │   │   │   │   │   │   │   │   │   ║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║   │   │   │   │   │   │   │   │   │   ║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║   │   │   │   │   │   │   │   │   │   ║
╚═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╝
searching for solution...
solution found:
╔═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╤═══╗
║ 20│ 45│ 32│ 19│ 44│ 57│ 18│ 43│ 56│ 17║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║ 74│ 13│ 22│ 73│ 14│ 23│ 61│ 15│ 24│ 60║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║ 33│ 30│  1│ 80│ 31│  2│ 85│ 58│  3│ 42║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║ 21│ 46│ 75│ 26│ 66│ 78│ 25│ 65│ 55│ 16║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║ 71│ 12│ 34│ 72│100│ 81│ 62│ 99│ 84│ 59║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║ 36│ 29│ 67│ 79│ 76│ 68│ 86│ 77│  4│ 41║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║  8│ 47│ 93│ 27│ 63│ 94│ 97│ 64│ 54│ 98║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║ 70│ 11│ 35│ 69│ 87│ 82│ 52│ 90│ 83│ 51║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║ 37│ 28│  7│ 38│ 96│  6│ 39│ 95│  5│ 40║
╟───┼───┼───┼───┼───┼───┼───┼───┼───┼───║
║  9│ 48│ 92│ 10│ 49│ 91│ 88│ 50│ 53│ 89║
╚═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╧═══╝
```
