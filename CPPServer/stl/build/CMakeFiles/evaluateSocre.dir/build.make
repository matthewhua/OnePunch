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
include CMakeFiles/evaluateSocre.dir/depend.make
# Include any dependencies generated by the compiler for this target.
include CMakeFiles/evaluateSocre.dir/compiler_depend.make

# Include the progress variables for this target.
include CMakeFiles/evaluateSocre.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/evaluateSocre.dir/flags.make

CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.o: CMakeFiles/evaluateSocre.dir/flags.make
CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.o: /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/evaluateSocre.cpp
CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.o: CMakeFiles/evaluateSocre.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building CXX object CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.o"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.o -MF CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.o.d -o CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.o -c /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/evaluateSocre.cpp

CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing CXX source to CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.i"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/evaluateSocre.cpp > CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.i

CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling CXX source to assembly CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.s"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/evaluateSocre.cpp -o CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.s

# Object files for target evaluateSocre
evaluateSocre_OBJECTS = \
"CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.o"

# External object files for target evaluateSocre
evaluateSocre_EXTERNAL_OBJECTS =

evaluateSocre: CMakeFiles/evaluateSocre.dir/evaluateSocre.cpp.o
evaluateSocre: CMakeFiles/evaluateSocre.dir/build.make
evaluateSocre: CMakeFiles/evaluateSocre.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking CXX executable evaluateSocre"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/evaluateSocre.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/evaluateSocre.dir/build: evaluateSocre
.PHONY : CMakeFiles/evaluateSocre.dir/build

CMakeFiles/evaluateSocre.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/evaluateSocre.dir/cmake_clean.cmake
.PHONY : CMakeFiles/evaluateSocre.dir/clean

CMakeFiles/evaluateSocre.dir/depend:
	cd /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles/evaluateSocre.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/evaluateSocre.dir/depend

