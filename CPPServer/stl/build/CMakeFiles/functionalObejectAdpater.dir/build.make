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
include CMakeFiles/functionalObejectAdpater.dir/depend.make
# Include any dependencies generated by the compiler for this target.
include CMakeFiles/functionalObejectAdpater.dir/compiler_depend.make

# Include the progress variables for this target.
include CMakeFiles/functionalObejectAdpater.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/functionalObejectAdpater.dir/flags.make

CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.o: CMakeFiles/functionalObejectAdpater.dir/flags.make
CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.o: /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/functionalObejectAdpater.cpp
CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.o: CMakeFiles/functionalObejectAdpater.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --progress-dir=/home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building CXX object CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.o"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.o -MF CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.o.d -o CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.o -c /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/functionalObejectAdpater.cpp

CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Preprocessing CXX source to CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.i"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/functionalObejectAdpater.cpp > CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.i

CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green "Compiling CXX source to assembly CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.s"
	/usr/bin/c++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo/functionalObejectAdpater.cpp -o CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.s

# Object files for target functionalObejectAdpater
functionalObejectAdpater_OBJECTS = \
"CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.o"

# External object files for target functionalObejectAdpater
functionalObejectAdpater_EXTERNAL_OBJECTS =

functionalObejectAdpater: CMakeFiles/functionalObejectAdpater.dir/functionalObejectAdpater.cpp.o
functionalObejectAdpater: CMakeFiles/functionalObejectAdpater.dir/build.make
functionalObejectAdpater: CMakeFiles/functionalObejectAdpater.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color --switch=$(COLOR) --green --bold --progress-dir=/home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Linking CXX executable functionalObejectAdpater"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/functionalObejectAdpater.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/functionalObejectAdpater.dir/build: functionalObejectAdpater
.PHONY : CMakeFiles/functionalObejectAdpater.dir/build

CMakeFiles/functionalObejectAdpater.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/functionalObejectAdpater.dir/cmake_clean.cmake
.PHONY : CMakeFiles/functionalObejectAdpater.dir/clean

CMakeFiles/functionalObejectAdpater.dir/depend:
	cd /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/STlDemo /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build /home/matthew/IdeaProjects/OnePunch/CPPServer/stl/build/CMakeFiles/functionalObejectAdpater.dir/DependInfo.cmake --color=$(COLOR)
.PHONY : CMakeFiles/functionalObejectAdpater.dir/depend
