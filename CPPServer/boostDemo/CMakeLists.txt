cmake_minimum_required(VERSION 3.14)

project(boostdemo)

file( GLOB APP_SOURCES RELATIVE ${CMAKE_CURRENT_SOURCE_DIR} *.cpp )

foreach( testsourcefile ${APP_SOURCES} )
    string( REPLACE ".cpp" "" testname ${testsourcefile} )
    add_executable( ${testname} ${testsourcefile} )
    set_target_properties(${testname} PROPERTIES LINKER_LANGUAGE CXX)
    if(OpenMP_CXX_FOUND)
        target_link_libraries(${testname})
    endif()
    install(TARGETS ${testname} DESTINATION "bin/ciphers")
endforeach(testsourcefile ${APP_SOURCES})

set(CMAKE_CXX_STANDARD 14)

if (WIN32)
    set(BOOST_ROOT D:/DevelopTOOLs/boost_1_77_0)
    find_package(Boost 1.77.0 COMPONENTS )
    set(Boost_DETAILED_FAILURE_MSG ON)
    set(Boost_DEBUG on)

    set(BOOST_INCLUDEDIR ${BOOST_ROOT})
    set(BOOST_LIBRARYDIR ${BOOST_ROOT})

    include_directories(${BOOST_ROOT})
#    target_link_libraries(boostdemo ${Boost_LIBRARIES})
endif ()


