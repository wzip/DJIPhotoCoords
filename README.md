# DJIPhotoCoords
Takes DJI drone photo coordinates from photo metadata (EXIF) and puts them in a table for plotting

This Jupyter notebook requires the Python Imaging Library (PIL) fork pillow.
To install in anaconda, at the anaconda prompt:

c:\> conda install pillow

To use this script, place the notebook (DJIPhotoCoords.ipynb) in the same folder
as the photos taken by a DJI drone and run the notebook in Jupyter.  The notebook
will create a CSV file (output.csv by default that can be changed) containing the
filenames, Latitudes (in Degrees Minutes Seconds), Longitudes (in Degrees Minutes
Seconds), drone altitude that the picture was taken in meters, and drone altitude
that the picture was taken in feet.  The CSV can be used to plot the photo
locations in a GIS, Google Earth, etc.

# Dependencies
Python version >= 3.7.11
Additional Python dependencies can be found in pyDependencies.txt

