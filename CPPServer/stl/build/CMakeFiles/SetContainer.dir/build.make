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
include CMakeFiles/SetContainer.dir/depend.make
# Include any dependencies generated by the compiler for this target.
include CMakeFiles/SetContainer.dir/compiler_depend.make

# Include the progress variables for this target.
include CMakeFiles/SetContainer.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/SetContainer.dir/flags.make

CMakeFiles/SetContainer.dir/SetContainer.cpp.o: CMakeFiles/SetContainer.dir/flags.make
CMakeFiles/SetContainer.dir/SetContainer.cpp.o: /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/SetContainer.cpp
CMakeFiles/SetContainer.dir/SetContainer.cpp.o: CMakeFiles/SetContainer.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building CXX object CMakeFiles/SetContainer.dir/SetContainer.cpp.o"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/SetContainer.dir/SetContainer.cpp.o -MF CMakeFiles/SetContainer.dir/SetContainer.cpp.o.d -o CMakeFiles/SetContainer.dir/SetContainer.cpp.o -c /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/SetContainer.cpp

CMakeFiles/SetContainer.dir/SetContainer.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing CXX source to CMakeFiles/SetContainer.dir/SetContainer.cpp.i"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/SetContainer.cpp > CMakeFiles/SetContainer.dir/SetContainer.cpp.i

CMakeFiles/SetContainer.dir/SetContainer.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling CXX source to assembly CMakeFiles/SetContainer.dir/SetContainer.cpp.s"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/SetContainer.cpp -o CMakeFiles/SetContainer.dir/SetContainer.cpp.s

# Object files for target SetContainer
SetContainer_OBJECTS = \
"CMakeFiles/SetContainer.dir/SetContainer.cpp.o"

# External object files for target SetContainer
SetContainer_EXTERNAL_OBJECTS =

SetContainer: CMakeFiles/SetContainer.dir/SetContainer.cpp.o
SetContainer: CMakeFiles/SetContainer.dir/build.make
SetContainer: CMakeFiles/SetContainer.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking CXX executable SetContainer"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/SetContainer.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/SetContainer.dir/build: SetContainer
.PHONY : CMakeFiles/SetContainer.dir/build

CMakeFiles/SetContainer.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/SetContainer.dir/cmake_clean.cmake
.PHONY : CMakeFiles/SetContainer.dir/clean

CMakeFiles/SetContainer.dir/depend:
	cd /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles/SetContainer.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/SetContainer.dir/depend

