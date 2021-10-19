//
// Created by Admin on 2021/8/25.
//

#include <functional>
#include <boost/lexical_cast.hpp>
using namespace std;
using namespace boost;

int main()
{
   int i = lexical_cast<int>("123");
   string s = lexical_cast<string>(1.23);
   printf("%s",s.c_str());
    return 0;
}