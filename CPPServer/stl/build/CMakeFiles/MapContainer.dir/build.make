# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.21

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:

#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:

# Disable VCS-based implicit rules.
% : %,v

# Disable VCS-based implicit rules.
% : RCS/%

# Disable VCS-based implicit rules.
% : RCS/%,v

# Disable VCS-based implicit rules.
% : SCCS/s.%

# Disable VCS-based implicit rules.
% : s.%

.SUFFIXES: .hpux_make_needs_suffix_list

# Command-line flag to silence nested $(MAKE).
$(VERBOSE)MAKESILENT = -s

#Suppress display of executed commands.
$(VERBOSE).SILENT:

# A target that is always out of date.
cmake_force:
.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /usr/bin/cmake

# The command to remove a file.
RM = /usr/bin/cmake -E rm -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build

# Include any dependencies generated for this target.
include CMakeFiles/MapContainer.dir/depend.make
# Include any dependencies generated by the compiler for this target.
include CMakeFiles/MapContainer.dir/compiler_depend.make

# Include the progress variables for this target.
include CMakeFiles/MapContainer.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/MapContainer.dir/flags.make

CMakeFiles/MapContainer.dir/MapContainer.cpp.o: CMakeFiles/MapContainer.dir/flags.make
CMakeFiles/MapContainer.dir/MapContainer.cpp.o: /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/MapContainer.cpp
CMakeFiles/MapContainer.dir/MapContainer.cpp.o: CMakeFiles/MapContainer.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building CXX object CMakeFiles/MapContainer.dir/MapContainer.cpp.o"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/MapContainer.dir/MapContainer.cpp.o -MF CMakeFiles/MapContainer.dir/MapContainer.cpp.o.d -o CMakeFiles/MapContainer.dir/MapContainer.cpp.o -c /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/MapContainer.cpp

CMakeFiles/MapContainer.dir/MapContainer.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing CXX source to CMakeFiles/MapContainer.dir/MapContainer.cpp.i"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/MapContainer.cpp > CMakeFiles/MapContainer.dir/MapContainer.cpp.i

CMakeFiles/MapContainer.dir/MapContainer.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling CXX source to assembly CMakeFiles/MapContainer.dir/MapContainer.cpp.s"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/MapContainer.cpp -o CMakeFiles/MapContainer.dir/MapContainer.cpp.s

# Object files for target MapContainer
MapContainer_OBJECTS = \
"CMakeFiles/MapContainer.dir/MapContainer.cpp.o"

# External object files for target MapContainer
MapContainer_EXTERNAL_OBJECTS =

MapContainer: CMakeFiles/MapContainer.dir/MapContainer.cpp.o
MapContainer: CMakeFiles/MapContainer.dir/build.make
MapContainer: CMakeFiles/MapContainer.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking CXX executable MapContainer"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/MapContainer.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/MapContainer.dir/build: MapContainer
.PHONY : CMakeFiles/MapContainer.dir/build

CMakeFiles/MapContainer.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/MapContainer.dir/cmake_clean.cmake
.PHONY : CMakeFiles/MapContainer.dir/clean

CMakeFiles/MapContainer.dir/depend:
	cd /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles/MapContainer.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/MapContainer.dir/depend
