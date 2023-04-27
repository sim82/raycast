// let A0;
// let A1;
// let A2;
// A0 = #{ id: 1, t: 30, next: A1 };
// A1 = #{ id: 2, t: 30, next: A2 };
// A2 = #{ id: 3, t: 30, next: A0 };

const B = 
#{	
	A0: #{ id: 1, t: 30, next: "A1" },
	A1: #{ id: 2, t: 30, next: "A2" },
	A2: #{ id: 3, t: 30, next: "A0" },
};
// let start = A0;
// fn start() {
// 	A0
// } 
// let start = || A0;
// x+1
#{ start: B }