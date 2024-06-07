# rbxlx-to-rojo

**This is a fork of the [original project](https://github.com/rojo-rbx/rbxlx-to-rojo) due to the original being unmaintained.**

Tool to convert existing Roblox games into Rojo projects by reading their `rbxl` or `rbxlx` place files.

# Using rbxlx-to-rojo

## Setup

Before you can use rbxlx-to-rojo, you need the following:

- At least Rojo 0.5.0 Alpha 12 or higher to use the tool.
- A rbxlx place file that at least has scripts

If there aren't any scripts in the rbxlx file, rbxlx-to-rojo will return an error.

Download the [source code](https://github.com/Striker2783/rbxlx-to-rojo/releases).

Compile it with GUI using

```
cargo build --release --features gui
```

In the `target/release` directory, the executable `rbxlx-to-rojo` is the file to run. You can move it anywhere on your system.

## Porting the game

Before you can port your game into Rojo projects, you need a place/model file. If you have an existing game that isn't exported:

- Go to studio, click on any place, and then click on File -> Save to file as.

- Create a folder and name it whatever you want.

### Steps to port the game:

1. Double-click on rbxlx-to-rojo on wherever you installed it.
2. Select the .rbxl file you saved earlier.
3. Now, select the folder that you just created.

If you followed the steps correctly, you should see something that looks like this:
![](assets/folders.png)

Congratulations, you successfully ported an existing game using rbxlx-to-rojo!

## License

rbxlx-to-rojo is available under The Mozilla Public License, Version 2. Details are available in [LICENSE.md](LICENSE.md).
