'''
This program creates a csv table with containing photo coordinates
extracted from exif tags on photos taken using DJI drones.  These
tags and photo ID's can be used to plot phot locations using tools
such as Geographic Information Systems, Google Earth, etc.

This is a simple version.  To run it just place DJIPhotoCoords.py
in the folder where with the photos you want to create a coordinate
table in and run the file.  The output file will be a csv file with
the name contained in the OUTFILE constant.

Dependencies:
pillow
'''
import csv
import os
#from the pillow package
from PIL import Image
from PIL.ExifTags import TAGS

def get_exif(fn):
    ret = {}
    i = Image.open(fn)
    info = i._getexif()
    for tag, value in info.items():
        decoded = TAGS.get(tag, tag)
        ret[decoded] = value
    return ret

#Convert degrees, minutes, seconds to decimal degrees
def dmsToDD(d,m,s):
    ret = 0.0
    dm = float(m)+float(s/60.0)
    dd = float(d)+float(dm/60)
    return dd

def main():
    #make a list of files in current directory
    flist = os.listdir()
    images = []

    #***THIS IS WHERE YOU CHANGE THE OUTPUT FILENAME!!!
    #output CSV filename; You can change the filename in '' here!
    OUTFILE = 'output.csv'

    #populate images[] with file paths
    for file in flist:
        if file.endswith('.jpg') or file.endswith('.JPG'):
            fpath = os.path.abspath(file)
            images.append(fpath)

    #open a csv outputfile
    outputFile = open(OUTFILE, 'w', newline='')
    wrtr = csv.writer(outputFile)
    #write the first row as column headers
    wrtr.writerow(['FileName','Latitude_DMS','Longitude_DMS','Latitude_DD','Longitude_DD','Altitude_m','Altitude_ft'])

    for image in images:
        imInfo = get_exif(image)
        fname = os.path.basename(image)
        #retrieve GPSInfo from photo metadata; dictionary with sub-tuples
        gpsInfo = imInfo['GPSInfo']
        #retrieve latitude DMS from GPSInfo from photo
        latDeg = float(gpsInfo[2][0])
        latMin = float(gpsInfo[2][1])
        latMinInt = int(latMin)
        latSec = float(gpsInfo[2][2])
        latDD = dmsToDD(latDeg,latMin,latSec)
        #make lat degrees negative if in S hemisphere (S of equator)
        if gpsInfo[1] == 'S':
            latDeg = latDeg*-1.0
            latDD = latDD*-1.0
            latDegInt = int(latDeg)
        #retrieve longitude DMS from GPSInfo from photo
        lonDeg = float(gpsInfo[4][0])
        lonMin = float(gpsInfo[4][1])
        lonMinInt = int(lonMin)
        lonSec = float(gpsInfo[4][2])
        lonDD = dmsToDD(lonDeg,lonMin,lonSec)
        #make lon degrees negative is W of prime meridian
        if gpsInfo[3] == 'W':
            lonDeg = lonDeg*-1.0
            lonDD = lonDD*-1.0
        lonDegInt = int(lonDeg)
        latDMS = str(latDegInt)+' '+str(latMinInt)+' '+str(latSec)
        lonDMS = str(lonDegInt)+' '+str(lonMinInt)+' '+str(lonSec)
        #retrieve altitude in meters
        altM = float(gpsInfo[6])
        altF = altM*3.28084
        wrtr.writerow([str(fname),latDMS,lonDMS,str(latDD),str(lonDD),str(altM),str(altF)])
    #end for loop
    outputFile.close()
    print('All finished! Output written to '+OUTFILE)

if __name__=='__main__':
    main()
