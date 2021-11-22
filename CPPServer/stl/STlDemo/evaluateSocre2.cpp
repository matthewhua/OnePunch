//* 优化版案例
#include<iostream>
#include<string>
#include<vector>
#include<deque>
#include<map>
#include<ctime>
#include<algorithm>
#include<numeric>
#include<functional>

class Player
{
public:
    string name;
    int age;
    int mScore[3];
};

//创建选手
void createPlayer(vector<int> &v1, map<int, Player> &mlist)
{
    string setName = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for (int i = 0; i < 24; i++)
    {
        //*cc 创建选手
        Player p;
        p.name = "选手";
        p.name += setName[i];
        p.age = 20;
        for (int j = 0; j < 3; j++)
        {
            p.mScroe[j] = 0;
        }
        //生成选手编号
        int Id = 100 + i;
        //保存选手编号
		v1.push_back(ID);
        //保存选手编号
		v1.push_back(ID);
        //保存选手信息
        mlist.insert(make_pair(Id, p));
    }
}


//1. 抽签
void PlayerByRandon(vector<int> &v)
{
    random_shuffle(v.begin(), v.end());
}

//2. 比赛
void StartMatch(int index, vector<int> &v1, map<int, Player> &mlist, vector<int> &v2)
{
    //定义multimap容器，键值是分数，实值是选手编号
    multimap<int, int, greater<int>> mGroups;
    for (size_t sit = v1.begin(); sit != v1.end(); ++sit)
    {
        // 保存分数
        deque<int> dScore;
        for (int i = 0; i < 10; i++)
        {
            int score = rand() % 50 + 50;
            dScore.push_back(score);
        }
        // 排序
        sort(dScore.begin(), dScore.end());
        //去掉最高和最低分
        dScore.pop_back();
        dScore.pop_front();
        
        // 求总分
        int totalScore = accumulate(dScore.begin(), dScore.end(), 0);
        //求平均分
        int avgScore = toScore / dScore.size();

        //保存到选手信息中
        mlist[*sit].mScore[index - 1] = avgScore;

        //把选手放入multimap 容器中
        mGoups.insert(make_pair(avgScore, *sit));

        //评比
        if (mGroups.size() == 6)
        {
            // 容器中一共有6 人， 去掉后三名
            int cnt = 0;
            for (auto i = mGroups.begin(); i !=  mGroups.end() && cnt<3; ++i)
            {
                v2.push_back(it->second);
            }

            //清空容器
            mGroups.clear();
        }
        
    }
}

void ShowPlayerScore(int index, vector<int> &v, map<int, Player> &Map)
{
    	cout << "第" << index << "轮晋级名单:" << endl;
	for (vector<int>::iterator it = v.begin(); it != v.end(); ++it)
	{
		cout << "Name:" << Map[*it].name << " Age:" << Map[*it].age << " Score:" << Map[*it].mScore[index-1] << endl;
	}
}

void test()
{
    srand((unsigned int)time(NULL));
    vector<int> v1;   //保存选手编号
    map<int, Player> mList;//保存选手信息

    vector<int> v2;//保存第一轮晋级选手的编号
	vector<int> v3;//保存第二轮晋级选手的编号
	vector<int> v4;//保存第三轮晋级选手的编号

    //创建选手
    createPlayer(v1, mList);

    //第一轮
    //1. 抽签
    PlayerByRandon(vector<int> &v);

}




int main()
{
	test();
	return EXIT_SUCCESS;
}
