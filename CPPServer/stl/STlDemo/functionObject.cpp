#include<iostream>
using namespace std;
#include<functional>
#include<algorithm>
#include<vector>

// 函数对象
void test()
{
    vector<int> v;
	v.push_back(8);
	v.push_back(1);
	v.push_back(6);
	v.push_back(3);
	v.push_back(7);

    sort(v.begin(), v.end(), greater<int>());
    for_each(v.begin(), v.end(),  [](int val){cout << val << " ";});
    //[](int val){cout << val << " "; }//匿名函数
}


int main()
{
    test();

	//system("pause"); windows only 
	return EXIT_SUCCESS;
}
