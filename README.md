# Superpatch

\#hype mod manager designed for [S.T.A.L.K.E.R Anomaly](https://www.moddb.com/mods/stalker-anomaly). Made in \#rust (\#blazinglyfast.) Much faster than Mod Organizer 2. It's also cross-platform. And it has some other features that I think are useful.

## Tools

Complete speculation. I haven't made them yet. I will! I promise!

#### Wine & Proton

If you use linux, detects already existing proton or wine. If it can't be found, helps you install it. Then makes a command to use it.

#### Modded EXEs

You'll probably need these. Download and add them.

#### MO2 Import

Import an existing modpack from MO2.

## Patching

I haven't done this yet.

Wondering why it's called Super"patch"? (terrible name btw.) It's because of this! I was originally just going to make a plugin for MO2. But it didn't seem feasible. So I made my own mod manager! In my opinion, S.T.A.L.K.E.R. modding suffers from constant CONFLICTS and PATCHES. It's really terrible. DLTX patches can be helpful, but they really seem like a bandaid on a dam. Many mods try to deal with conflicts by patching over files and implementing their new behavior AND the mod that they PRESUME you have's intended behavior. What if you have THREE mods that modify the same file? Well, you better hope one of them implements the behavior of all three in a patch. This can get much more out of hand. So! that is where this comes in hand. 

You (will in the future soon be able to and do by doing this) can select a file with conflicts in the File menu and open it in the Patch menu. Now, you can see a list of all the various sources for the files from many mods. You can also see DLTX patches! And then you can like... see the hashes of the files and then like make a master patch or something that... it's still in progress. See PLAN.md if you really care about my ideas.

## Transferability

This one works right now. 

If you've installed things using the interface and not messed with them (please), moving the entire folder won't affect the instance. This also means you can share modpacks! Kind of.

Here's a folder structure for your reference.

- superpatch - This executable has no dependencies! You need it to use it. Duh.
- config/ - This folder holds all your settings and the instance DATA. You'll definitely need this.
- mods/ - This folder holds all the files for your mods. You'll definitely need this. Don't seperate this from config.
- patch/ - This folder holds all the files for your patch. If you have a patch, you'll need this. Don't seperate this from config.
- archives/ - This folder holds all the original archives for your mods. You don't nesscarily need this, but it's useful for reinstalling things.  
- instance/ - While nothing requires you to, I reccommend you install your game in here. You have to make this folder. It's your responsibility.
- .saved/ - This is YOUR save data. I would reccomend keeping it. But you probably don't need to share it with others.
- .vfs/ - This is the instance's temporary folder where the game actually runs. You don't need to share this. You can also probably delete this at will, as it's generated on launch! With exception. This MAY contain your save data if new files appear and the changes have not been saved to .saved/. To avoid this, make sure to Save VFS Changes in the File menu before deleting it. Also, don't delete it while the game is running. Duh.

Now, that's a little complicated. I should probably make another project for sharing modpacks.. And installing superpatch. And installing the game with it. Put that in the list.

## Install

You probably want an install! Uhh.. Give me some time. It's not done.

## A note on this project

This was made in rust, a language that I have never used before. Why? Why not! I am quite terrible with organizing code in any language, so as of the time of writing, there is ONE file of code. 2000 lines and counting.

Neither am I the greatest at issue management. All of the current issues that I know about are in the source code, either in TODO or FIXME blocks. This will probably be replaced by Github issues once this project is in a fully-functional state, but for now, don't expect the greatest help.

I have definitely used some AI for this. I haven't coded in a fair while, so I went through the cycle of the latest tech. Copilot, Kilo, Claude, blah blah blah. I didn't end up using these much. These uses are not labeled, but I think pretty minimal. I have finally settled it down to an art. I mostly write the code myself. If there is something that doesn't work, I may paste it into a seperate chatbot, and poke it for a fix. This is unnanotated. If there is a function that is just busywork, I ask a seperate chatbot to write it. If it looks good and works, I add it. This is annotated, however, with "SLOP". \#ethical \#ethicalslop \#homemade

I don't intend to update this software much after it gets into a reliable state. Tools could be an exception. This is just another one of my projects throughout my journey across the web. I haven't even played that much S.T.A.L.K.E.R. Maybe... two hours? I don't know. I just was like, "this is unacceptable" and decided to make my own mod manager. So this could be complete garbage. But I'm pretty sure it's not. At least hopefully the mod management part can be useful to someone.