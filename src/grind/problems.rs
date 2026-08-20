//! Built-in bank of classic problems for grind mode. Works fully offline —
//! no leetcode session or database required.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GrindProblem {
    pub title: &'static str,
    pub difficulty: Difficulty,
    pub language: &'static str,
    pub code: &'static str,
}

pub const PROBLEMS: &[GrindProblem] = &[
    GrindProblem {
        title: "Two Sum",
        difficulty: Difficulty::Easy,
        language: "python",
        code: "def two_sum(nums, target):\n    seen = {}\n    for i, n in enumerate(nums):\n        rest = target - n\n        if rest in seen:\n            return [seen[rest], i]\n        seen[n] = i\n    return []",
    },
    GrindProblem {
        title: "Climbing Stairs",
        difficulty: Difficulty::Easy,
        language: "python",
        code: "def climb_stairs(n):\n    a, b = 1, 1\n    for _ in range(n - 1):\n        a, b = b, a + b\n    return b",
    },
    GrindProblem {
        title: "Best Time to Buy and Sell Stock",
        difficulty: Difficulty::Easy,
        language: "python",
        code: "def max_profit(prices):\n    low = float('inf')\n    best = 0\n    for p in prices:\n        low = min(low, p)\n        best = max(best, p - low)\n    return best",
    },
    GrindProblem {
        title: "Valid Parentheses",
        difficulty: Difficulty::Easy,
        language: "python",
        code: "def is_valid(s):\n    pairs = {')': '(', ']': '[', '}': '{'}\n    stack = []\n    for ch in s:\n        if ch in pairs:\n            if not stack or stack.pop() != pairs[ch]:\n                return False\n        else:\n            stack.append(ch)\n    return not stack",
    },
    GrindProblem {
        title: "Reverse Linked List",
        difficulty: Difficulty::Easy,
        language: "python",
        code: "def reverse_list(head):\n    prev = None\n    while head:\n        nxt = head.next\n        head.next = prev\n        prev = head\n        head = nxt\n    return prev",
    },
    GrindProblem {
        title: "Binary Search",
        difficulty: Difficulty::Easy,
        language: "python",
        code: "def search(nums, target):\n    lo, hi = 0, len(nums) - 1\n    while lo <= hi:\n        mid = (lo + hi) // 2\n        if nums[mid] == target:\n            return mid\n        if nums[mid] < target:\n            lo = mid + 1\n        else:\n            hi = mid - 1\n    return -1",
    },
    GrindProblem {
        title: "Maximum Subarray",
        difficulty: Difficulty::Medium,
        language: "python",
        code: "def max_subarray(nums):\n    best = cur = nums[0]\n    for n in nums[1:]:\n        cur = max(n, cur + n)\n        best = max(best, cur)\n    return best",
    },
    GrindProblem {
        title: "Merge Intervals",
        difficulty: Difficulty::Medium,
        language: "python",
        code: "def merge(intervals):\n    intervals.sort()\n    out = []\n    for lo, hi in intervals:\n        if out and lo <= out[-1][1]:\n            out[-1][1] = max(out[-1][1], hi)\n        else:\n            out.append([lo, hi])\n    return out",
    },
    GrindProblem {
        title: "Longest Substring Without Repeating",
        difficulty: Difficulty::Medium,
        language: "python",
        code: "def length_of_longest(s):\n    seen = {}\n    best = start = 0\n    for i, ch in enumerate(s):\n        if ch in seen and seen[ch] >= start:\n            start = seen[ch] + 1\n        seen[ch] = i\n        best = max(best, i - start + 1)\n    return best",
    },
    GrindProblem {
        title: "Trapping Rain Water",
        difficulty: Difficulty::Hard,
        language: "python",
        code: "def trap(height):\n    lo, hi = 0, len(height) - 1\n    lmax = rmax = water = 0\n    while lo < hi:\n        if height[lo] < height[hi]:\n            lmax = max(lmax, height[lo])\n            water += lmax - height[lo]\n            lo += 1\n        else:\n            rmax = max(rmax, height[hi])\n            water += rmax - height[hi]\n            hi -= 1\n    return water",
    },
];
