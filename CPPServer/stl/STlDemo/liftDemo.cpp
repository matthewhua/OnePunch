//
// Created by Admin on 2021/11/15.
//
#include<iostream>
using namespace std;
#include<list>
#include<vector>
#include<queue>
#include<string>
#include<ctime>

//抽象人员
class Student
{
public:
    string name;
};

// 打印人员
void printVector(vector<Student> &vec)
{
    for (auto & it : vec) {
        cout << it.name << endl;
    }
}

//创建人员
void CreateStudent(queue<Student> &que, int num)
{
    string setName = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    int sum = rand() % 10;
    for (int i = 0; i < sum; i++) {
        Student stu;
        char buf[64] = { 0 };
        sprintf(buf, "%d", num);
        string s(buf);

        stu.name = "第";
        stu.name += s;
        stu.name += "层";
        stu.name += setName[i];

        que.push(stu);  // 每层的人员放入队列容器
    }
}

//mylist是电梯,que队列容器,pushV是拷贝进电梯的人员
int PushList(list<Student> &mylist, queue<Student> &que, vector<Student> &pushV)
{
    int tmpPush = 0; //临时变量，记录出电梯人员数
    
    while (!que.empty())
    {
        if (mylist.size() >= 15) //电梯满了
            break;

        Student s = que.front();

        //拷贝到vector
        pushV.push_back(s);

        //进电梯
        mylist.push_back(s);

        //队列中队头元素出容器
        que.pop();

        tmpPush++;
    }
    return tmpPush;
}

//mylist是电梯,popV记录出电梯人员，num层数
int PopList(list<Student> &myList, vector<Student> &popV, int num)
{
    int tmppop = 0;
    if (num == 17)
    {
        while (!myList.empty())
        {
            Student s = myList.front();
            // 把电梯的人拷贝到popV 中
            popV.push_back(s);

            myList.pop_front();
            tmppop++;
        }
    }

    int n = rand() % 5;//随机出电梯人数
    if (n = 0)
        return tmppop;
    //当电梯有人，且人数大于等于随机出电梯的人数才让人出电梯
    
    if(myList.size() > 0 && myList.size() >= n)
    {
        for (int i = 0; i < n; i++) {
            Student student = myList.front();
            //把出电梯的人员拷贝到popV中
            popV.push_back(student);
            myList.pop_front();//移除电梯的人
            tmppop++;
        }
    }

    return tmppop;
}

void test()
{
    srand((unsigned int)time(NULL));
    list<Student> lift; //英式电梯

    int PushNum = 0; //记录进电梯的总人数
    int PopNum = 0; //记录出电梯的总人数

    vector<Student> pushV;//记录进电梯的人员
    vector<Student> popV; //记录出电梯的人员

    //电梯上升
    for (int i = 1; i <= 24; i++) {

        //创建人员
        queue<Student> que;
        //创建人员函数
        CreateStudent(que, i);
        if (lift.size() <= 15)
        {
            //到24层事件结束,
            if (i < 24)
            {
                //进电梯
                PushNum += PushList(lift, que, pushV);
            }
        }

        //判断出电梯条件
        if (lift.size() > 0) //电梯要有人才能出
        {
            if (i > 1)  //1 层时, 电梯是空的
            {
                // 出电梯
                PopNum += PopList(lift, popV, i);
            }
        }
    }

    //打印进电梯的人员
    printVector(pushV);
    cout << "进电梯人数: " << PushNum << endl;
    // 打印出电梯的人员
    printVector(popV);
    cout << "出电梯人数: " << PopNum << endl;

}

int main()
{
    test();
    system("pause");
    return EXIT_SUCCESS;
}



