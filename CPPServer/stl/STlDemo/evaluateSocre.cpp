//
// Created by Admin on 2021/11/15.
//
#include <iostream>
#include <vector>
#include <deque>
#include <string>
#include <algorithm>//�㷨ͷ�ļ�
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
        Student stu; //����ѧ��
        stu.name = "ѧ��";
        stu.name += setName[i];
        stu.mScore = 0;
        vstu.push_back(stu);
    }
}


void EvaluateScore(vector<Student> &vstu) {
    srand((unsigned int) time(NULL));

    //����ѧ��
    for (auto it = vstu.begin(); it != vstu.end(); ++it) {
        // �������
        deque<int> dScore;
        //��ί��ѧ�����
        for (int i = 0; i < 10; i++) {
            int socre = rand() % 70 + 30;
            dScore.push_back(socre);
        }
        //����
        sort(dScore.begin(), dScore.end()); // �Ѿ������
        // ȥ����߷ֺ���ͷ�
        dScore.pop_back();
        dScore.pop_front();

        // ���ܷ�
        int total = 0;
        for (deque<int>::iterator sit = dScore.begin(); sit != dScore.end(); ++sit) {
            total += (*sit);
        }

        // ��ƽ����
        int averageScore = total / dScore.size();

        // ƽ���ִ洢��������
        it->mScore = averageScore;
    }
}

bool myCompare(Student &s1, Student &s2)
{
    return s1.mScore > s2.mScore;
}

//3.��������ӡ
void ShowStudentScore(vector<Student> &vstu)
{
    sort(vstu.begin(), vstu.end(), myCompare);

    for (auto item = vstu.begin(); item != vstu.end() ; ++item) {
        cout << "Name:" << item->name << "Socre:" << item->mScore << endl;
    }
}


void test()
{
    //�洢ѧ��������
    vector<Student>  vstu;

	//1.����ѧ��
	CreateStudent(vstu);
    //2.��ί��ѧ�����
    EvaluateScore(vstu);

    //3.��������ӡ
    ShowStudentScore(vstu);
}

int main()
{
    test();
    system("pause");
    return EXIT_SUCCESS;
}


