//
// Created by Admin on 2021/11/15.
//
#include <iostream>
#include <vector>
#include <deque>
#include <string>
#include <algorithm>//算法头文件
#include <ctime>

using namespace std;

class Student
{
public:
    string name;
    int mScore;
};

void CreateStudent(vector<Student> &vstu)
{
    string setName = "ABCDE";
    for (int i = 0; i < 5; ++i) {
        Student stu; //创建学生
        stu.name = "学生";
        stu.name += setName[i];
        stu.mScore = 0;
        vstu.push_back(stu);
    }
}


void EvaluateScore(vector<Student> &vstu) {
    srand((unsigned int) time(NULL));

    //遍历学生
    for (auto it = vstu.begin(); it != vstu.end(); ++it) {
        // 保存分数
        deque<int> dScore;
        //评委给学生打分
        for (int i = 0; i < 10; i++) {
            int socre = rand() % 70 + 30;
            dScore.push_back(socre);
        }
        //排序
        sort(dScore.begin(), dScore.end()); // 已经排序好
        // 去掉最高分和最低分
        dScore.pop_back();
        dScore.pop_front();

        // 求总分
        int total = 0;
        for (deque<int>::iterator sit = dScore.begin(); sit != dScore.end(); ++sit) {
            total += (*sit);
        }

        // 求平均分
        int averageScore = total / dScore.size();

        // 平均分存储到对象中
        it->mScore = averageScore;
    }
}

bool myCompare(Student &s1, Student &s2)
{
    return s1.mScore > s2.mScore;
}

//3.排名并打印
void ShowStudentScore(vector<Student> &vstu)
{
    sort(vstu.begin(), vstu.end(), myCompare);

    for (auto item = vstu.begin(); item != vstu.end() ; ++item) {
        cout << "Name:" << item->name << "Socre:" << item->mScore << endl;
    }
}


void test()
{
    //存储学生的容器
    vector<Student>  vstu;

	//1.创建学生
	CreateStudent(vstu);
    //2.评委给学生打分
    EvaluateScore(vstu);

    //3.排名并打印
    ShowStudentScore(vstu);
}

int main()
{
    test();
    system("pause");
    return EXIT_SUCCESS;
}


