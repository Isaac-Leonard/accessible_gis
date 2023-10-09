# What is this:
This is a application to make GIS data somewhat accessible to blind people.
I am blind and am currently studying a bachelor of science majoring in earth science.
As part of my degree I must take several courses on GIS, aka working with satellite data, maps and other geographic information.
Up until now much of this has been purely inaccessible, blind people can't exactly look at images and even for non image data most existing GIS apps have put in minimal effort for making the textual parts of there interfaces work with screen readers (appart from maybe arcgis, but that costs like $20,000 and only works on windows).
This project goes a small way to changing that.
The goal is to be a simple application for working with GIS data primarily through audio and text.
It will currently only work on Macs, as that is what I personally use but maybe one day I may be able to make it cross platform.
It is written in rust as that is my preferred language, it uses the Cacao bindings to Appkit and makes use of my own custom UI framework for Cacao written largely for use in this project and the rust bindings to GDAL for reading and writing files.
Currently to use this you must have rust and GDAL installed.
## What works
Currently you can load in both vector and raster data from files.
For vector data you can look at the attribute tables for each shape in the dataset and can look at what type of shape each table corresponds to.
For raster data you can listen to a graph of the image.
It drops the resolution of the specified image down, turns each pixel value into a frequency and then plays each row of the image through your headphones, left to right, top to bottom.
There are settings you can change to change the maximum and minimum frequencies of the played audio, a greater difference between these will allow you to pick out more minor differences in pixel values.
There are also settings for controlling the duration each row will play for and how many columns and rows the image resolution will be set to.
A similar function is in place for playing the histogram of images with byte valued data for spectral analysis.
It will create a histogram, run through it and set a frequency value for each pixel count then play the frequencies from left to right as an audio graph, modeled off of the sonify R package and graphs that exist in some IOS apps such as the ones for looking at battery usage throughout the day in the iphones settings, again there are settings to control aspects of this graph.
It will also provide stats for raster images and spatial reference information for all datasets when possible.
