import zipfile
import os
import sys

def make_liaison_fmu(model_name, dll_path, xml_path, output_dir, platform='x86_64-windows'):
    output_fmu_path = os.path.join(output_dir, model_name + ".fmu")

    with zipfile.ZipFile(output_fmu_path, 'w', zipfile.ZIP_DEFLATED) as fmu:
        # Define the binary folder structure inside the FMU
        binaries_dir = os.path.join('binaries', platform)
        
        # Change the name of the DLL file to match the model_name
        new_dll_name = model_name + ".dll"
        dll_arcname = os.path.join(binaries_dir, new_dll_name)
        
        # Add the renamed DLL file to the FMU inside the binaries/platform directory
        fmu.write(dll_path, arcname=dll_arcname)
        
        # Add the modelDescription.xml file to the FMU at the root
        fmu.write(xml_path, arcname='modelDescription.xml')
    
    print(f"FMU created at {output_fmu_path}")

if __name__ == "__main__":
    if len(sys.argv) != 5:
        print("Usage: python pack_fmu.py <model_name> <path_to_dll> <path_to_modelDescription.xml> <output_directory>")
        sys.exit(1)

    model_name = sys.argv[1]
    dll_path = sys.argv[2]
    xml_path = sys.argv[3]
    output_dir = sys.argv[4]

    make_liaison_fmu(model_name, dll_path, xml_path, output_dir)
