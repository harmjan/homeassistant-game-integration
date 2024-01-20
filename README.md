# Dark souls remastered Home Assistant integration
This program looks at the frames received from a capture device and detects if the YOU DIED words are printed on the screen like they are in Dark Souls Remastered. If so a webhook in [Home Assistant](https://www.home-assistant.io/) is called so you can add integrations like flashing the lights in the room red.

In my setup I'm playing the game on a steam deck, I'm capturing the game with an NZXT Signal HD60 in between the steam deck and a screen. This program runs on another computer that talks with the capture device to get a stream of frames of the game.
