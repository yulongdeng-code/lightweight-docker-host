import random
from typing import List, Dict, Optional
from dataclasses import dataclass


@dataclass
class PromptCandidate:
    prompt: str
    scores: Dict[str, float]
    traces: List[dict]
    generation: int
    parent_ids: List[str]


class GEPAEngine:
    def __init__(self, dspy_module=None, budget: int = 100):
        self.dspy = dspy_module
        self.budget = budget
        self.pool: List[PromptCandidate] = []
        self.pareto_front: Dict[str, PromptCandidate] = {}
        
    def run(self, initial_prompt: str, train_set: List[dict]) -> str:
        """主进化循环"""
        self.pool = [PromptCandidate(
            prompt=initial_prompt,
            scores={},
            traces=[],
            generation=0,
            parent_ids=[]
        )]
        
        for step in range(self.budget):
            parents = self._select_from_pareto_frontier()
            
            if random.random() < 0.7:
                child = self._reflective_mutate(parents[0])
            else:
                child = self._reflective_crossover(parents)
            
            traces = self._run(child.prompt, random.sample(train_set, min(2, len(train_set))))
            scores = self._score(traces)
            
            child.scores = scores
            child.traces = traces
            self.pool.append(child)
            self._update_pareto(child)
        
        return self._get_best_prompt()
    
    def _select_from_pareto_frontier(self) -> List[PromptCandidate]:
        """从Pareto前沿选择父代"""
        if not self.pareto_front:
            return random.sample(self.pool, min(2, len(self.pool)))
        candidates = list(self.pareto_front.values())
        return random.sample(candidates, min(2, len(candidates)))
    
    def _reflective_mutate(self, parent: PromptCandidate) -> PromptCandidate:
        """基于执行轨迹的反思变异"""
        reflection = f"Improve this prompt for better results: {parent.prompt}"
        mutated = f"{parent.prompt}\n\nNote: {reflection}"
        return PromptCandidate(
            prompt=mutated,
            scores={},
            traces=[],
            generation=parent.generation + 1,
            parent_ids=[str(id(parent))]
        )
    
    def _reflective_crossover(self, parents: List[PromptCandidate]) -> PromptCandidate:
        """基于反思的交叉"""
        combined = "\n\n".join([p.prompt for p in parents])
        return PromptCandidate(
            prompt=combined,
            scores={},
            traces=[],
            generation=max(p.generation for p in parents) + 1,
            parent_ids=[str(id(p)) for p in parents]
        )
    
    def _run(self, prompt: str, tasks: List[dict]) -> List[dict]:
        """运行提示词并收集轨迹"""
        traces = []
        for task in tasks:
            traces.append({"task": task, "success": random.random() > 0.3})
        return traces
    
    def _score(self, traces: List[dict]) -> Dict[str, float]:
        """评估执行轨迹"""
        success_rate = sum(1 for t in traces if t.get("success")) / max(len(traces), 1)
        return {
            "accuracy": success_rate,
            "efficiency": random.random(),
            "relevance": random.random()
        }
    
    def _update_pareto(self, candidate: PromptCandidate):
        """更新Pareto前沿"""
        key = str(id(candidate))
        dominated = False
        
        for existing in list(self.pareto_front.values()):
            if self._dominates(existing, candidate):
                dominated = True
                break
        
        if not dominated:
            self.pareto_front[key] = candidate
            self.pareto_front = {
                k: v for k, v in self.pareto_front.items()
                if not any(self._dominates(cand, v) and cand != v for cand in self.pareto_front.values())
            }
    
    def _dominates(self, a: PromptCandidate, b: PromptCandidate) -> bool:
        """检查a是否支配b"""
        a_scores = a.scores
        b_scores = b.scores
        
        if not a_scores or not b_scores:
            return False
            
        better_in_all = all(a_scores.get(k, 0) >= b_scores.get(k, 0) for k in set(a_scores.keys()) | set(b_scores.keys()))
        better_in_one = any(a_scores.get(k, 0) > b_scores.get(k, 0) for k in set(a_scores.keys()) | set(b_scores.keys()))
        return better_in_all and better_in_one
    
    def _get_best_prompt(self) -> str:
        """获取最佳提示词"""
        if not self.pool:
            return ""
        best = max(self.pool, key=lambda c: sum(c.scores.values()) if c.scores else 0)
        return best.prompt


class SkillExtractor:
    """技能自动提取器"""
    def extract_skill(self, successful_traces: List[dict]) -> dict:
        """从成功轨迹中提取可复用的技能"""
        return {
            "name": "extracted_skill",
            "description": "Auto-extracted skill from successful execution",
            "code": "# Skill implementation",
            "examples": successful_traces[:3]
        }


class SelfEvolutionEngine:
    """自我进化引擎"""
    def __init__(self):
        self.skills: List[dict] = []
        self.generation = 0
    
    def evolve(self, feedback: dict):
        """基于反馈进行五阶段进化"""
        self.generation += 1
        return {
            "new_skills": self._evolve_skills(feedback),
            "new_tools": self._evolve_tools(feedback),
            "new_prompts": self._evolve_prompts(feedback),
            "code_changes": self._evolve_code(feedback),
            "monitor_updates": self._evolve_monitoring(feedback)
        }
    
    def _evolve_skills(self, feedback: dict) -> List[dict]:
        return []
    
    def _evolve_tools(self, feedback: dict) -> List[dict]:
        return []
    
    def _evolve_prompts(self, feedback: dict) -> List[str]:
        return []
    
    def _evolve_code(self, feedback: dict) -> List[str]:
        return []
    
    def _evolve_monitoring(self, feedback: dict) -> List[dict]:
        return []
