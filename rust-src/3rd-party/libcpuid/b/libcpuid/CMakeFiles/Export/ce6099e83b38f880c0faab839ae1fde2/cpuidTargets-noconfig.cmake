#----------------------------------------------------------------
# Generated CMake target import file.
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "cpuid::cpuid" for configuration ""
set_property(TARGET cpuid::cpuid APPEND PROPERTY IMPORTED_CONFIGURATIONS NOCONFIG)
set_target_properties(cpuid::cpuid PROPERTIES
  IMPORTED_IMPLIB_NOCONFIG "${_IMPORT_PREFIX}/lib/libcpuid.dll.a"
  IMPORTED_LOCATION_NOCONFIG "${_IMPORT_PREFIX}/bin/libcpuid.dll"
  )

list(APPEND _cmake_import_check_targets cpuid::cpuid )
list(APPEND _cmake_import_check_files_for_cpuid::cpuid "${_IMPORT_PREFIX}/lib/libcpuid.dll.a" "${_IMPORT_PREFIX}/bin/libcpuid.dll" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
