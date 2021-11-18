//
// Created by Admin on 2021/11/17.
//

//对组

#include<iostream>
using namespace std;
#include<string>

int main()
{
    pair<string, int> myp("matthew", 18);

    cout << myp.first << " " << myp.second << endl;
    pair<int, string> myp2(1,"悟空");
    cout << myp2.first << " " << myp2.second << endl;

    system("pause");
}