#William Zipse 2022
#Updated 2025 with AI assistance
import os
import csv
import tkinter as tk
from tkinter import filedialog, messagebox
from PIL import Image
from PIL.ExifTags import TAGS

class DJIPhotoCoordsGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("DJI Photo Coordinates Extractor")
        self.root.geometry("600x400")
        
        # Variables
        self.input_folder = tk.StringVar()
        self.output_file = tk.StringVar()
        
        # Create and pack widgets
        self.create_widgets()
        
    def create_widgets(self):
        # Input Folder Selection
        tk.Label(self.root, text="Input Folder:").pack(pady=5)
        folder_frame = tk.Frame(self.root)
        folder_frame.pack(fill=tk.X, padx=10)
        
        tk.Entry(folder_frame, textvariable=self.input_folder, width=50).pack(side=tk.LEFT, padx=5)
        tk.Button(folder_frame, text="Browse", command=self.browse_input_folder).pack(side=tk.LEFT)
        
        # Output File Selection
        tk.Label(self.root, text="Output CSV File:").pack(pady=5)
        output_frame = tk.Frame(self.root)
        output_frame.pack(fill=tk.X, padx=10)
        
        tk.Entry(output_frame, textvariable=self.output_file, width=50).pack(side=tk.LEFT, padx=5)
        tk.Button(output_frame, text="Browse", command=self.browse_output_file).pack(side=tk.LEFT)
        
        # Process Button
        tk.Button(self.root, text="Process Photos", command=self.process_photos, 
                 bg="#4CAF50", fg="white", padx=20, pady=10).pack(pady=20)
        
        # Status Label
        self.status_label = tk.Label(self.root, text="")
        self.status_label.pack(pady=10)
        
    def browse_input_folder(self):
        folder_selected = filedialog.askdirectory()
        if folder_selected:
            self.input_folder.set(folder_selected)
            
    def browse_output_file(self):
        file_selected = filedialog.asksaveasfilename(
            defaultextension=".csv",
            filetypes=[("CSV files", "*.csv"), ("All files", "*.*")]
        )
        if file_selected:
            self.output_file.set(file_selected)
            
    def get_exif(self, fn):
        ret = {}
        i = Image.open(fn)
        info = i._getexif()
        for tag, value in info.items():
            decoded = TAGS.get(tag, tag)
            ret[decoded] = value
        return ret

    def dmsToDD(self, d, m, s):
        dm = float(m) + float(s/60.0)
        dd = float(d) + float(dm/60)
        return dd
        
    def process_photos(self):
        input_folder = self.input_folder.get()
        output_file = self.output_file.get()
        
        if not input_folder or not output_file:
            messagebox.showerror("Error", "Please select both input folder and output file")
            return
            
        try:
            # Get list of JPG files
            images = []
            for file in os.listdir(input_folder):
                if file.lower().endswith('.jpg'):
                    fpath = os.path.join(input_folder, file)
                    images.append(fpath)
                    
            if not images:
                messagebox.showwarning("Warning", "No JPG files found in the selected folder")
                return
                
            # Process files
            with open(output_file, 'w', newline='') as outputFile:
                wrtr = csv.writer(outputFile)
                wrtr.writerow(['FileName','Latitude_DMS','Longitude_DMS','Latitude_DD','Longitude_DD','Altitude_m','Altitude_ft'])
                
                for image in images:
                    try:
                        imInfo = self.get_exif(image)
                        fname = os.path.basename(image)
                        gpsInfo = imInfo['GPSInfo']
                        
                        # Process latitude
                        latDeg = float(gpsInfo[2][0])
                        latMin = float(gpsInfo[2][1])
                        latMinInt = int(latMin)
                        latSec = float(gpsInfo[2][2])
                        latDD = self.dmsToDD(latDeg, latMin, latSec)
                        if gpsInfo[1] == 'S':
                            latDeg = latDeg*-1.0
                            latDD = latDD*-1.0
                        latDegInt = int(latDeg)
                        
                        # Process longitude
                        lonDeg = float(gpsInfo[4][0])
                        lonMin = float(gpsInfo[4][1])
                        lonMinInt = int(lonMin)
                        lonSec = float(gpsInfo[4][2])
                        lonDD = self.dmsToDD(lonDeg, lonMin, lonSec)
                        if gpsInfo[3] == 'W':
                            lonDeg = lonDeg*-1.0
                            lonDD = lonDD*-1.0
                        lonDegInt = int(lonDeg)
                        
                        latDMS = f"{latDegInt} {latMinInt} {latSec}"
                        lonDMS = f"{lonDegInt} {lonMinInt} {lonSec}"
                        
                        # Process altitude
                        altM = float(gpsInfo[6])
                        altF = altM*3.28084
                        
                        wrtr.writerow([str(fname), latDMS, lonDMS, str(latDD), str(lonDD), str(altM), str(altF)])
                        
                    except Exception as e:
                        print(f"Error processing {image}: {str(e)}")
                        continue
                        
            messagebox.showinfo("Success", f"Processing complete!\n{len(images)} photos processed.\nOutput saved to: {output_file}")
            self.status_label.config(text=f"Processed {len(images)} photos")
            
        except Exception as e:
            messagebox.showerror("Error", f"An error occurred: {str(e)}")

if __name__ == "__main__":
    root = tk.Tk()
    app = DJIPhotoCoordsGUI(root)
    root.mainloop() 