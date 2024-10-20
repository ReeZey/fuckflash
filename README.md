# fuckflash  
  
hacky tool to get old **Macromedia Director's Projector Skeleton** games out of its awful fullscreen mode 

example:
| before | after |
| -------- | ------- |  
| ![before image](https://raw.githubusercontent.com/reezey/fuckflash/master/before.png?raw=true) | ![after image](https://raw.githubusercontent.com/reezey/fuckflash/master/after.png?raw=true) |  

## tested & supported games  
- Bygg Bilar med Mulle Meck
- Onsalakorv
- Pettson & Findus i Trädgården  

and probably more!

## how to use
drag the exe ontop of the loader PANG it works, maybe  

## current limitations
you can only launch **one** games cause the DLL is loaded into the exe, blocking it from being loaded again

## how work
consists of two parts    
- fuckflash.dll > the dll file with detours    
- loader.exe > the dll injector/loader  
  

# build  
use 32-bit x86 assembly cause most games are 32-bit, its better to build for that architecture  
> `cargo b -r --target i686-pc-windows-msvc`  