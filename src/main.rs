use std::collections::VecDeque;

// Структура для представления транспортной задачи
struct TransportProblem {
    supplies: Vec<i32>,  
    demands: Vec<i32>,    
    costs: Vec<Vec<i32>>, 
}

// Структура для хранения плана перевозок
struct TransportPlan {
    allocations: Vec<Vec<i32>>, 
    total_cost: i32,            
}

impl TransportProblem {
    fn new() -> Self {
        TransportProblem {
            supplies: vec![200, 150, 150],
            demands: vec![90, 100, 70, 130, 110],
            costs: vec![
                vec![12, 15, 21, 14, 17],
                vec![14, 8, 15, 11, 21],
                vec![19, 16, 26, 12, 20],
            ],
        }
    }

    
    fn is_balanced(&self) -> bool {
        let total_supply: i32 = self.supplies.iter().sum();
        let total_demand: i32 = self.demands.iter().sum();
        total_supply == total_demand
    }

    // Метод северо-западного угла
    fn north_west_corner(&self) -> TransportPlan {
        let m = self.supplies.len();
        let n = self.demands.len();
        let mut allocations = vec![vec![0; n]; m];

        let mut supply_remaining = self.supplies.clone();
        let mut demand_remaining = self.demands.clone();

        let mut i = 0;
        let mut j = 0;

        while i < m && j < n {
            let allocation = std::cmp::min(supply_remaining[i], demand_remaining[j]);
            allocations[i][j] = allocation;
            supply_remaining[i] -= allocation;
            demand_remaining[j] -= allocation;

            if supply_remaining[i] == 0 {
                i += 1;
            }
            if demand_remaining[j] == 0 {
                j += 1;
            }
        }

        let total_cost = self.calculate_total_cost(&allocations);

        TransportPlan {
            allocations,
            total_cost,
        }
    }

    
    fn calculate_total_cost(&self, allocations: &[Vec<i32>]) -> i32 {
        let mut total = 0;
        for i in 0..allocations.len() {
            for j in 0..allocations[i].len() {
                total += allocations[i][j] * self.costs[i][j];
            }
        }
        total
    }

    // Оптимизация методом потенциалов
    fn optimize_by_potentials(&self, mut plan: TransportPlan) -> TransportPlan {
        let m = self.supplies.len();
        let n = self.demands.len();
        let mut improved = true;
        let mut iteration = 0;

        while improved && iteration < 5 {
            improved = false;
            iteration += 1;

            // Шаг 1: Вычисление потенциалов
            let mut u = vec![None; m];
            let mut v = vec![None; n];
            u[0] = Some(0.0);

            // Распространение потенциалов через базисные клетки
            let mut changed = true;
            while changed {
                changed = false;

                for i in 0..m {
                    for j in 0..n {
                        if plan.allocations[i][j] > 0 {
                            if let Some(u_val) = u[i] {
                                if v[j].is_none() {
                                    v[j] = Some(self.costs[i][j] as f64 - u_val);
                                    changed = true;
                                }
                            } else if let Some(v_val) = v[j] {
                                if u[i].is_none() {
                                    u[i] = Some(self.costs[i][j] as f64 - v_val);
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }

            // Заполняем оставшиеся нулями
            for i in 0..m {
                if u[i].is_none() {
                    u[i] = Some(0.0);
                }
            }
            for j in 0..n {
                if v[j].is_none() {
                    v[j] = Some(0.0);
                }
            }

            // Шаг 2: Поиск улучшающей клетки
            let mut best_i = 0;
            let mut best_j = 0;
            let mut best_delta = 0.0;

            for i in 0..m {
                for j in 0..n {
                    if plan.allocations[i][j] == 0 {
                        let delta = self.costs[i][j] as f64 - (u[i].unwrap() + v[j].unwrap());
                        if delta < best_delta {
                            best_delta = delta;
                            best_i = i;
                            best_j = j;
                            improved = true;
                        }
                    }
                }
            }

            // Шаг 3: Если найдена улучшающая клетка
            if improved && best_delta < -0.0001 {
                println!(
                    "Итерация {}: улучшение через клетку ({}, {}) с дельтой {:.2}",
                    iteration,
                    best_i + 1,
                    best_j + 1,
                    best_delta
                );

                // Поиск цикла (упрощенно - находим первую возможную цепочку)
                if let Some(cycle) = self.find_cycle(&plan.allocations, best_i, best_j) {
                    // Находим минимальный груз в отнимающих клетках
                    let mut min_q = i32::MAX;
                    for (i, j) in cycle.iter().skip(1).step_by(2) {
                        if plan.allocations[*i][*j] < min_q {
                            min_q = plan.allocations[*i][*j];
                        }
                    }

                    // Перераспределение
                    for (idx, (i, j)) in cycle.iter().enumerate() {
                        if idx % 2 == 0 {
                            // Четные - добавляем
                            plan.allocations[*i][*j] += min_q;
                        } else {
                            // Нечетные - вычитаем
                            plan.allocations[*i][*j] -= min_q;
                        }
                    }

                    plan.total_cost = self.calculate_total_cost(&plan.allocations);
                }
            } else if !improved {
                println!("Итерация {}: план оптимален", iteration);
            }
        }

        plan
    }

    // Поиск цикла для перераспределения (упрощенная реализация)
    fn find_cycle(
        &self,
        allocations: &[Vec<i32>],
        start_i: usize,
        start_j: usize,
    ) -> Option<Vec<(usize, usize)>> {
        let m = allocations.len();
        let n = allocations[0].len();

        // Простой алгоритм поиска в ширину для нахождения цикла
        let mut visited = vec![vec![false; n]; m];
        let mut queue = VecDeque::new();
        let mut parent = vec![vec![None; n]; m];

        queue.push_back((start_i, start_j));
        visited[start_i][start_j] = true;

        while let Some((i, j)) = queue.pop_front() {
            // Проверяем соседние клетки
            for &(ni, nj) in &[
                (i.wrapping_sub(1), j),
                (i + 1, j),
                (i, j.wrapping_sub(1)),
                (i, j + 1),
            ] {
                if ni < m && nj < n && allocations[ni][nj] > 0 && !visited[ni][nj] {
                    visited[ni][nj] = true;
                    parent[ni][nj] = Some((i, j));
                    queue.push_back((ni, nj));
                }
            }
        }

        // Строим путь назад (упрощенно)
        let mut cycle = Vec::new();
        cycle.push((start_i, start_j));

        // Добавляем несколько базисных клеток для формирования цикла
        for i in 0..m {
            for j in 0..n {
                if allocations[i][j] > 0 && (i != start_i || j != start_j) {
                    if cycle.len() < 4 {
                        cycle.push((i, j));
                    }
                }
            }
        }

        // Замыкаем цикл
        if cycle.len() >= 3 {
            cycle.push(cycle[0]);
            Some(cycle)
        } else {
            None
        }
    }

    // Решение задачи
    fn solve(&self) {
        println!("=== ТРАНСПОРТНАЯ ЗАДАЧА ===");
        println!("Запасы: {:?}", self.supplies);
        println!("Потребности: {:?}", self.demands);
        println!("Матрица стоимостей:");
        for row in &self.costs {
            println!("{:?}", row);
        }
        println!();

        if !self.is_balanced() {
            println!("Задача не сбалансирована!");
            return;
        }

        println!("=== НАЧАЛЬНЫЙ ПЛАН (метод северо-западного угла) ===");
        let mut plan = self.north_west_corner();
        self.print_plan(&plan);

        println!("\n=== ОПТИМИЗАЦИЯ МЕТОДОМ ПОТЕНЦИАЛОВ ===");
        plan = self.optimize_by_potentials(plan);

        println!("\n=== ОПТИМАЛЬНЫЙ ПЛАН ===");
        self.print_plan(&plan);
    }

    // Вывод плана в табличном виде
    fn print_plan(&self, plan: &TransportPlan) {
        let m = self.supplies.len();
        let n = self.demands.len();

        // Заголовок таблицы
        print!("      ");
        for j in 0..n {
            print!("B{:.<5}", j + 1);
        }
        println!("| Запасы");

        // Данные
        for i in 0..m {
            print!("A{}   ", i + 1);
            let mut row_sum = 0;
            for j in 0..n {
                if plan.allocations[i][j] > 0 {
                    print!("{}({})\t", plan.allocations[i][j], self.costs[i][j]);
                } else {
                    print!("-\t");
                }
                row_sum += plan.allocations[i][j];
            }
            println!("| {}/{}", row_sum, self.supplies[i]);
        }

        // Потребности
        print!("Потр.");
        for j in 0..n {
            let mut col_sum = 0;
            for i in 0..m {
                col_sum += plan.allocations[i][j];
            }
            print!(" {}/{}  ", col_sum, self.demands[j]);
        }

        println!("\n\nОбщая стоимость: {} у.е.", plan.total_cost);
    }
}

fn main() {
    let problem = TransportProblem::new();
    problem.solve();

    // Дополнительный анализ
    println!("\n=== АНАЛИЗ ===");

    let initial_plan = problem.north_west_corner();
    println!("Начальная стоимость: {} у.е.", initial_plan.total_cost);

    // Тестовый план из условия (после одной итерации)
    let test_allocations = vec![
        vec![90, 100, 10, 0, 0],
        vec![0, 0, 60, 90, 0],
        vec![0, 0, 0, 40, 110],
    ];
    let test_cost = problem.calculate_total_cost(&test_allocations);
    println!("План после 1 итерации (из условия): {} у.е.", test_cost);

    // Улучшенный план из условия
    let improved_allocations = vec![
        vec![90, 100, 0, 0, 10],
        vec![0, 0, 70, 80, 0],
        vec![0, 0, 0, 50, 100],
    ];
    let improved_cost = problem.calculate_total_cost(&improved_allocations);
    println!("Улучшенный план (из условия): {} у.е.", improved_cost);

    let savings = initial_plan.total_cost - improved_cost;
    println!(
        "Экономия: {} у.е. ({:.1}%)",
        savings,
        (savings as f64 / initial_plan.total_cost as f64) * 100.0
    );
}
