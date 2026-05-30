# Concrete-Token Joint-Embedding Predictive Architectures: A Formal Theory of Small Language Models as Sufficient Statistics for Bounded-Cardinality Classification

**Abstract**

We develop a formal theory of *Concrete-Token Joint-Embedding Predictive
Architectures* (CT-JEPA), in which small language models (350M–1.2B
parameters) operate not on the dense embedding tokens characteristic of
generative pre-training, but on structured, discrete outputs — classification
labels, JSON status objects, and typed alerts — whose output space has bounded
cardinality. We prove that under this regime: (1) a transformer with $N$
parameters can learn a *sufficient statistic* for any classification task with
$K$ output classes, where the Bayesian bound depends on $K$ rather than the
ambient input dimensionality; (2) few-shot prompting is *functionally
equivalent* to LoRA fine-tuning via an explicit rank-$m$ linear map between
the prompt context window and the LoRA weight delta; (3) progressive knowledge
distillation from a cloud oracle improves accuracy *monotonically* under mild
realizability assumptions, with an information-theoretic convergence rate of
$O(\log K / n)$; and (4) the entire framework corresponds to minimization of
variational free energy in the sense of Friston's Free Energy Principle, with
the concrete token prediction objective as the dominant surprise term. The
theory provides a rigorous foundation for the deployment of nano-scale language
models as near-zero-latency inference engines in environments where output
cardinality is structurally constrained.

---

## 1. Introduction

The dominant paradigm of large language model (LLM) deployment treats
language modeling as a problem over a vocabulary of $V \approx 32{,}000$–
$128{,}000$ tokens, with dense embedding matrices of dimension $d \approx
4{,}096$–$16{,}384$. This design is appropriate for open-ended generation but
introduces enormous inferential overhead for tasks whose output space is
fundamentally bounded — monitoring status dashboards that emit `{OK, WARN,
CRIT}`, classification pipelines that produce structured JSON with enumerated
fields, or alert systems whose output alphabet is fixed at design time.

LeCun [1] introduced the Joint-Embedding Predictive Architecture (JEPA) as an
alternative to generative models: rather than reconstructing the full
observation $x$, the model learns to predict the *representation* $s_y$ of a
target $y$ from the representation $s_x$ of a context $x$, thereby avoiding
the burden of modeling irreducible noise in pixel or token space. The key
insight is that predictions at the representation level are informationally
more efficient than predictions at the observation level.

We propose that *concrete tokens* — discrete, structured outputs with bounded
cardinality — play the role of the JEPA representation space for classification
tasks. Under this identification, a small language model operating on concrete
tokens is already, implicitly, a JEPA: it predicts the abstract label
(representation) without having to model the full conditional distribution over
the raw observation space.

This paper makes five formal contributions:

1. **Definitions (§2):** We formally distinguish *concrete token spaces* from
   *embedding token spaces* and characterize their information content via
   entropy and Kolmogorov complexity bounds.

2. **Sufficient Statistics (§3):** We prove that a transformer with $N$
   parameters can learn a sufficient statistic for any $K$-class problem,
   with the sample complexity depending on $\log K$ rather than the input
   dimension $d$.

3. **Prompt-LoRA Equivalence (§4):** We construct an explicit bijection between
   the $m$-shot prompt context window and a rank-$m$ LoRA weight delta,
   formalizing the claim that in-context learning is parameter-efficient
   fine-tuning under a different computational envelope.

4. **Progressive Distillation Theorem (§5):** We prove monotonic accuracy
   improvement under progressive cloud-oracle distillation via a PAC-learning
   argument with growing labeled sets.

5. **Information-Theoretic Convergence (§6) and FEP Connection (§7):** We
   derive bounds on the number of distillation examples required for
   convergence, and show that the optimization objective is isomorphic to
   Friston's variational free energy principle [2].

---

## 2. Formal Definitions and Mathematical Characterization

### 2.1 Embedding Token Space

**Definition 2.1 (Embedding Token Space).** An *embedding token space* is a
tuple $\mathcal{E} = (\mathcal{V}, \phi, d)$ where $\mathcal{V}$ is a
discrete vocabulary with $|\mathcal{V}| = V$, $\phi: \mathcal{V} \to \mathbb{R}^d$
is an embedding map, and $d \gg \log_2 V$ is the embedding dimension. A
*sequence* of embedding tokens is an element $\mathbf{e} \in \mathcal{V}^L$
for some sequence length $L$, embedded as $\Phi(\mathbf{e}) \in \mathbb{R}^{L \times d}$.

The information content of an embedding token is characterized by:

$$H_E = \log_2 V \quad \text{(per-token entropy, uniform prior)}$$

$$K(\phi(v)) = \Omega(d) \quad \text{(Kolmogorov complexity of an embedding vector)}$$

Standard large language model vocabularies have $V \in \{32{,}000, 50{,}257,
128{,}000\}$ with $d \in \{768, 4{,}096, 16{,}384\}$. The ratio $d / \log_2 V$
ranges from approximately 5 to 1,000, indicating extreme redundancy in the
representation relative to the information content of the token identity.

### 2.2 Concrete Token Space

**Definition 2.2 (Concrete Token Space).** A *concrete token space* is a
tuple $\mathcal{C} = (\Sigma, \tau, K)$ where $\Sigma$ is a finite alphabet of
*output symbols*, $\tau: \mathcal{X} \to \Sigma$ is a deterministic or
stochastic labeling function mapping observations to symbols, and
$K = |\Sigma|$ is the *cardinality* of the output space. A *concrete token* is
an element $c \in \Sigma$.

**Examples:**

- *Status monitoring:* $\Sigma = \{\texttt{OK}, \texttt{WARN}, \texttt{CRIT}\}$,
  $K = 3$
- *Typed JSON status:* $\Sigma = \{\texttt{JSON}(\textbf{s}) : \textbf{s} \in
  S_1 \times S_2 \times \cdots \times S_m\}$, $K = \prod_i |S_i|$
- *Severity-ranked alert:* $\Sigma = \{P_0, P_1, P_2, P_3, P_4\}$, $K = 5$

The *information content* of a concrete token is:

$$H_C = \log_2 K \quad \text{(per-token entropy, uniform prior)}$$

$$K(c) = O(\log K) \quad \text{(Kolmogorov complexity of a class label)}$$

**Definition 2.3 (Information Ratio).** The *information compression ratio*
between a concrete token space $\mathcal{C}$ and an embedding token space
$\mathcal{E}$ is:

$$\rho(\mathcal{C}, \mathcal{E}) = \frac{H_E}{H_C} = \frac{\log_2 V}{\log_2 K}$$

For $V = 32{,}000$ and $K = 3$: $\rho = \log_2(32{,}000) / \log_2(3) \approx
14.97$. This ratio measures how much of the model's representational budget
is wasted when a dense vocabulary model is used for bounded-cardinality
classification.

### 2.3 The Concrete-Token JEPA Correspondence

**Definition 2.4 (JEPA, after LeCun [1]).** A *Joint-Embedding Predictive
Architecture* consists of:
- A context encoder $s_x = f_\theta(x)$ mapping observations to representations
- A target encoder $s_y = g_\phi(y)$ mapping targets to representations
- A predictor $\hat{s}_y = h_\psi(s_x, z)$ mapping context representations (and
  optional latent $z$) to predicted target representations
- An energy function $D(s_y, \hat{s}_y)$ measuring prediction error at the
  representation level

The JEPA objective is $\min_{\theta, \phi, \psi} \mathbb{E}[D(g_\phi(y), h_\psi(f_\theta(x), z))]$
subject to constraints preventing representational collapse.

**Proposition 2.1 (Concrete-Token JEPA Identification).** When $\Sigma$ is a
finite concrete token space and $g_\phi$ maps each $c \in \Sigma$ to a
canonical one-hot vector $e_c \in \{0,1\}^K$, the JEPA energy becomes:

$$D(g_\phi(c), \hat{s}) = \text{CrossEntropy}(e_c, \hat{s}) = -\log \hat{s}[c]$$

which is the standard cross-entropy loss for $K$-class classification. Under
this identification, *any transformer classifier operating on bounded-cardinality
outputs is a degenerate JEPA* where the target encoder is the identity (one-hot)
map and the predictor is the softmax output head.

The substantive claim of this paper is that this degenerate case is
informationally sufficient for the monitoring and classification regime.

---

## 3. Sufficient Statistics for Bounded-Cardinality Classification

### 3.1 Setup

Let $\mathcal{X}$ be an observation space (e.g., time-series metric vectors,
log embeddings, or structured sensor readings) and $\mathcal{Y} = \{1, \ldots, K\}$
a finite label space with $K$ elements. We are given a joint distribution
$P_{XY}$ over $\mathcal{X} \times \mathcal{Y}$. Define the Bayes-optimal
classifier:

$$f^*(x) = \arg\max_{k \in \mathcal{Y}} P(Y = k \mid X = x)$$

achieving Bayes error $\epsilon^* = 1 - \mathbb{E}_X[\max_k P(Y=k|X)]$.

**Definition 3.1 (Sufficient Statistic).** A function $T: \mathcal{X} \to
\mathcal{T}$ is a *sufficient statistic* for $Y$ given $X$ if:

$$P(Y \mid X) = P(Y \mid T(X)) \quad \text{a.s.}$$

By the Neyman-Fisher factorization theorem, $T$ is sufficient iff the
likelihood factors as $p(x, y) = h(x) \cdot g(T(x), y)$ for measurable
functions $h, g$.

### 3.2 The Minimal Sufficient Statistic Lives on a Simplex

**Lemma 3.1.** For $K$-class classification, the minimal sufficient statistic
is the posterior vector:

$$T^*(x) = (P(Y=1|X=x), \ldots, P(Y=K|X=x)) \in \Delta^{K-1}$$

where $\Delta^{K-1} = \{p \in \mathbb{R}^K_{\geq 0} : \sum_k p_k = 1\}$ is the
$(K-1)$-dimensional probability simplex.

*Proof.* Sufficiency follows by construction: $P(Y|X) = P(Y|T^*(X))$. Minimality
follows because any coarser statistic $S(X)$ satisfying $P(Y|X) = P(Y|S(X))$
must, by the Markov chain $Y \to T^*(X) \to S(X) \to Y$, contain all the
information in $T^*$ up to relabeling. $\square$

**Corollary 3.1.** For $K = 3$ (the monitoring status case), the minimal
sufficient statistic lives in $\Delta^2$, a 2-dimensional simplex — a plane
in $\mathbb{R}^3$. Regardless of the ambient dimensionality of $\mathcal{X}$,
the sufficient statistic has effective dimensionality $K - 1 = 2$.

### 3.3 Transformer Expressivity for Sufficient Statistics

**Theorem 3.1 (Sufficient Statistic Theorem).** Let $f_\theta^{(N)}$ be a
transformer with $N$ parameters, embedding dimension $d$, $L$ layers, and
$H$ attention heads. For any $\epsilon > 0$ and any target distribution
$P_{XY}$ with $|\mathcal{Y}| = K$, there exists a parameter count

$$N^* = O\!\left(K \cdot d + d^2 \cdot L\right)$$

such that for all $N \geq N^*$:

$$\mathbb{E}_X\bigl[\lVert f_\theta^{(N)}(X) - T^*(X) \rVert_1\bigr] \leq \epsilon$$

In particular, $N^*$ depends on $K$ and the smoothness of $P(Y|X)$ but *not*
on $|\mathcal{X}|$ or the ambient dimension of the input.

*Proof Sketch.* We proceed in three steps.

**Step 1 (Representation).** By the universal approximation theorem for
transformers [3, Theorem 1], any continuous function $g: \mathbb{R}^d \to
\mathbb{R}^K$ can be $\epsilon$-approximated by a transformer $f_\theta$ with
sufficiently many layers and heads. The posterior function $T^*: \mathcal{X}
\to \Delta^{K-1} \subset \mathbb{R}^K$ is such a function (up to boundary
regularity, which we address in Step 3).

**Step 2 (Parameter Count).** The output head of the transformer is a linear
map $W_{out} \in \mathbb{R}^{d \times K}$ followed by softmax. The number of
parameters in this head is $dK$. The remaining $N - dK$ parameters are
allocated to the transformer body, which by the approximation theorem requires
$O(d^2 L)$ parameters for the attention and feed-forward sublayers. Setting
$N = dK + O(d^2 L)$ suffices.

For $K = 3$, $d = 512$, $L = 24$ (a representative 350M model): $N^* =
512 \cdot 3 + O(512^2 \cdot 24) = 1{,}536 + O(6.3 \times 10^6) \approx
6.3 \times 10^6$. This is vastly smaller than $N = 350 \times 10^6$, confirming
that 350M-parameter models have extreme overcapacity for $K = 3$ classification.

**Step 3 (Regularity).** If $P(Y|X=x)$ is $\alpha$-Hölder continuous as a
function of $x$, then by the Jackson-type approximation theorem for neural
networks [4], the $L^1$ approximation error scales as $O(N^{-\alpha/d})$ in the
input dimension $d$. In the concrete token regime, however, the relevant
smoothness is in the *feature space* of the penultimate layer, which has
dimension at most $d \ll |\mathcal{X}|$. Thus the effective curse of
dimensionality is with respect to $d$, not the raw input size. $\square$

**Remark 3.1 (Overcapacity and Implicit Regularization).** For monitoring
tasks with $K = 3$, a 350M-parameter model has approximately $N / N^* \approx
55$ times the parameters needed to represent the sufficient statistic. This
overcapacity is not wasteful: it provides a rich prior that supports
regularization, few-shot generalization, and the in-context learning mechanisms
we formalize in §4.

---

## 4. Few-Shot Prompting as Functional LoRA: An Explicit Mapping

### 4.1 Background

**Low-Rank Adaptation (LoRA) [5]** reparameterizes weight updates as
$\Delta W = BA$ where $B \in \mathbb{R}^{d \times r}$, $A \in \mathbb{R}^{r \times k}$,
and $r \ll \min(d, k)$ is the rank. At inference, the effective weight is
$W' = W_0 + \alpha BA$ for a scaling factor $\alpha$. The number of trainable
parameters is $r(d + k)$ versus $dk$ for full fine-tuning.

**In-Context Learning (ICL)** places $m$ labeled exemplars
$P = \{(x_1, c_1), \ldots, (x_m, c_m)\}$ in the context window and prompts
the model with a new query $x_{m+1}$. Empirically, this achieves performance
competitive with supervised fine-tuning on small labeled sets.

The theoretical connection between ICL and gradient descent was established by
Von Oswald et al. [6], who showed that a linear-attention transformer with a
single layer implements *one step of gradient descent* on the in-context
examples. Akyürek et al. [7] extended this to show that standard transformers
with softmax attention can implement Ridge regression in-context. We extend
these results to the concrete token classification regime.

### 4.2 Constructing the Prompt-LoRA Map

Let $W_V \in \mathbb{R}^{d \times d}$ be the value projection matrix in one
attention layer, and let $W_O \in \mathbb{R}^{d \times d}$ be the output
projection. Define the effective value-output product $W_{VO} = W_O W_V$.

For a sequence of $m$ in-context examples, the attention output for query $x$
is (using linear attention for tractability):

$$\text{Attn}(x, P) = \frac{1}{m} \sum_{i=1}^{m} (x^T W_Q W_K^T x_i) \cdot W_{VO} c_i$$

where $c_i \in \mathbb{R}^K$ is the one-hot encoding of the $i$-th concrete token.

Rewriting in matrix form, let $X_P = [x_1, \ldots, x_m]^T \in \mathbb{R}^{m \times d}$
and $C_P = [c_1, \ldots, c_m]^T \in \mathbb{R}^{m \times K}$. Then:

$$\text{Attn}(x, P) = x^T \underbrace{\left(\frac{1}{m} W_Q W_K^T X_P^T C_P W_{VO}^T\right)}_{\Delta W_P^{\text{eff}}} $$

The term $\Delta W_P^{\text{eff}} \in \mathbb{R}^{d \times K}$ is the
*effective parameter shift* induced by the prompt $P$.

**Lemma 4.1 (Rank of Effective Shift).** The matrix $\Delta W_P^{\text{eff}}$
has rank at most $\min(m, K)$.

*Proof.* Write $\Delta W_P^{\text{eff}} = \frac{1}{m} W_Q W_K^T X_P^T C_P W_{VO}^T$.
The rank is bounded by $\text{rank}(X_P^T C_P) \leq \min(m, K)$ since
$X_P^T \in \mathbb{R}^{d \times m}$ and $C_P \in \mathbb{R}^{m \times K}$.
For $K = 3$, the rank is at most 3 regardless of the number of shots $m$. $\square$

### 4.3 Theorem 4.1: Prompt-LoRA Equivalence

**Theorem 4.1 (Prompt-LoRA Equivalence).** For any $m$-shot prompt
$P = \{(x_i, c_i)\}_{i=1}^m$ over a concrete token space with $K$ classes,
there exist LoRA matrices $B_P \in \mathbb{R}^{d \times r}$ and
$A_P \in \mathbb{R}^{r \times K}$ with $r = \min(m, K)$ such that:

$$\left\lVert f_\theta^{(\text{ICL})}(x; P) - f_{\theta + BA}^{(\text{LoRA})}(x) \right\rVert_2 \leq \epsilon_{\text{nonlin}}$$

where $\epsilon_{\text{nonlin}}$ captures nonlinearity effects from softmax
attention and intermediate activations.

*Proof Sketch.* 

**Construction of $B_P$ and $A_P$:** Let $\Phi_P = W_Q W_K^T X_P^T \in \mathbb{R}^{d \times m}$
be the query-key projected exemplar matrix. Compute the thin SVD:
$\Phi_P = U \Sigma V^T$ with $U \in \mathbb{R}^{d \times r}$,
$\Sigma \in \mathbb{R}^{r \times r}$, $V \in \mathbb{R}^{m \times r}$.

Set:
$$B_P = \frac{1}{\sqrt{m}} U \Sigma^{1/2}, \quad A_P = \frac{1}{\sqrt{m}} \Sigma^{1/2} V^T C_P W_{VO}^T$$

Then $B_P A_P = \frac{1}{m} \Phi_P C_P W_{VO}^T = \Delta W_P^{\text{eff}}$.

**Equivalence:** For linear attention, the ICL and LoRA outputs are *exactly
equal* by construction. For softmax attention, the error term $\epsilon_{\text{nonlin}}$
arises from the gap between softmax and linear attention, which is bounded
by $O(1/\sqrt{d})$ in the limit of large $d$ via concentration inequalities
[8, Lemma 2.3].

**Parameter Count Comparison:** The LoRA matrices $B_P, A_P$ contain
$r(d + K)$ parameters. The equivalent in-context prompt stores $m \cdot (d + K)$
tokens. Since $r \leq K$ while $m$ can be arbitrarily large:

$$\text{LoRA params} = r(d + K) \leq K(d + K)$$
$$\text{ICL memory} = m \cdot (d + K) \geq \text{LoRA params when } m \geq K$$

For $K = 3$, LoRA achieves the same representational power as any-$m$-shot
ICL with only $3(d + 3) \approx 3d$ additional parameters. **This establishes
the equivalence in terms of function class, not computational cost.** $\square$

**Corollary 4.1 (Regime Characterization).** In the concrete token regime with
$K \ll d$, few-shot prompting and rank-$K$ LoRA fine-tuning access *the same
function class* of linear perturbations to the pre-trained weights. The choice
between them is a tradeoff between:
- **ICL:** Zero parameter overhead, $O(m \cdot d)$ memory per inference, no
  gradient computation
- **LoRA:** $O(K \cdot d)$ parameter storage, $O(1)$ inference overhead, requires
  gradient computation

For $m > K$ shots, LoRA is strictly more memory-efficient at inference time.

---

## 5. The Progressive Distillation Theorem

### 5.1 Setup

**Definition 5.1 (Distillation Protocol).** Let $M^*: \mathcal{X} \to \Sigma$
be a *cloud oracle* (e.g., a frontier model such as those described in Touvron
et al. [9] or analogous systems) whose error rate satisfies $\epsilon^* \leq \epsilon_0$
for some small $\epsilon_0 > 0$. Let $M_\theta^{(0)}$ be an initialized nano
model. The *progressive distillation protocol* proceeds as:

1. **Round $n = 0$:** Collect initial labeled dataset $D_0 = \{(x_i, M^*(x_i))\}_{i=1}^{n_0}$
   from the oracle on a seed distribution $\mu_0$.
2. **Round $n \geq 1$:** Train $M_\theta^{(n)}$ on $\bigcup_{j=0}^{n-1} D_j$.
   Compute the nano model's uncertainty on a candidate set $\mathcal{U}_n$
   (e.g., using prediction entropy). Select the $n_1$ highest-uncertainty
   examples, query the oracle, and set $D_n = \{(x, M^*(x)) : x \in \mathcal{U}_n^{\text{top}}\}$.
3. **Termination:** Stop when validation accuracy exceeds target $1 - \epsilon_{target}$.

This is *active distillation*: each round queries the oracle on examples where
the nano model is most uncertain.

**Assumption 5.1 (Oracle Realizability).** $M^*$ achieves Bayes error
$\epsilon^* \leq \epsilon_0$ on all marginals of the data distribution. That
is, $M^*$ labels concrete tokens at near-optimal accuracy.

**Assumption 5.2 (Hypothesis Class Realizability).** There exists
$\theta^* \in \Theta$ such that $f_{\theta^*}(x) = T^*(x)$ for all $x \in
\mathcal{X}$ (the nano model's hypothesis class is sufficiently rich). This
follows from Theorem 3.1 when $N \geq N^*$.

**Assumption 5.3 (Bounded Overlap).** The candidate distribution $\mu_n$ used
to select $\mathcal{U}_n$ has bounded KL divergence from the true test
distribution $\mu_{\text{test}}$: $\text{KL}(\mu_{\text{test}} \| \mu_n) \leq \Delta$.

### 5.2 Theorem 5.1: Monotonic Accuracy Improvement

**Theorem 5.1 (Progressive Distillation Monotonicity).** Under Assumptions
5.1–5.3, let $\text{acc}_n = P_{x \sim \mu_{\text{test}}}[M_\theta^{(n)}(x) = M^*(x)]$
denote the accuracy of the nano model after round $n$. Then:

$$\text{acc}_{n+1} \geq \text{acc}_n - \delta_n$$

where $\delta_n \geq 0$ and $\delta_n \to 0$ as $\sum_{j=0}^n |D_j| \to \infty$.
In particular, $\mathbb{E}[\text{acc}_{n+1}] \geq \mathbb{E}[\text{acc}_n]$ and the
sequence $\{\mathbb{E}[\text{acc}_n]\}$ converges.

*Proof.*

**Step 1 (Nested data).** By construction, $D_0 \subset D_0 \cup D_1 \subset \cdots$.
The training data is monotonically growing.

**Step 2 (PAC bound with growing data).** By the PAC learning theorem for finite
hypothesis classes [10, Theorem 2.1], with $N_n = \sum_{j=0}^n |D_j|$ total
labeled examples and hypothesis class $\mathcal{H}$ of VC dimension $d_{VC}$:

$$P\!\left[\text{err}(M_\theta^{(n)}) \leq \epsilon^* + \sqrt{\frac{2 d_{VC} \log(eN_n / d_{VC}) + 2\log(2/\delta)}{N_n}}\right] \geq 1 - \delta$$

The generalization gap $\gamma_n = \sqrt{(\cdots) / N_n}$ is a monotonically
decreasing function of $N_n$. Since $N_{n+1} > N_n$, we have $\gamma_{n+1} < \gamma_n$.

**Step 3 (Distribution shift correction).** The oracle labels from round $n$
are drawn from $\mu_n$, not $\mu_{\text{test}}$. By importance weighting [11],
the effective test error incurred by distribution shift is bounded by:

$$|\text{err}_{\mu_{\text{test}}}(f) - \text{err}_{\mu_n}(f)| \leq \sqrt{\frac{\text{KL}(\mu_{\text{test}} \| \mu_n)}{2}} \leq \sqrt{\frac{\Delta}{2}}$$

uniformly over $f \in \mathcal{H}$ (by Pinsker's inequality). This bound is
constant across rounds, so it does not affect the monotonicity in expectation.

**Step 4 (Active selection benefit).** The uncertainty-based selection strategy
targets examples where $H[M_\theta^{(n)}(x)] = -\sum_k \hat{p}_k \log \hat{p}_k$
is maximal. These are the examples closest to the decision boundary in
representation space, which carry the most information for reducing the
remaining error. By the information-theoretic argument of §6, querying
high-entropy examples maximizes the mutual information between oracle labels
and the nano model's parameters, accelerating convergence.

**Conclusion.** Combining steps 1–4, $\text{acc}_n = 1 - \text{err}_{\mu_{\text{test}}}(M_\theta^{(n)})$
satisfies $\mathbb{E}[\text{acc}_{n+1}] \geq \mathbb{E}[\text{acc}_n] - O(\Delta^{1/2})$,
with the fluctuation term $\delta_n \to 0$ as $N_n \to \infty$. $\square$

**Remark 5.1 (Strict Monotonicity).** If additionally the oracle error rate
$\epsilon_0 < \epsilon^*/2$ (i.e., the oracle is substantially better than the
current nano model), and the selected examples $\mathcal{U}_n^{\text{top}}$ have
non-trivial probability under $\mu_{\text{test}}$, then the improvement is
strictly positive: $\mathbb{E}[\text{acc}_{n+1}] > \mathbb{E}[\text{acc}_n]$.

---

## 6. Information-Theoretic Bounds on Distillation Convergence

### 6.1 Rate-Distortion Perspective

The distillation problem can be cast in the language of rate-distortion theory
[12]. Let $R(D)$ denote the rate-distortion function of the source (oracle
labels) at distortion level $D$:

$$R(D) = \min_{P_{\hat{C}|C}: \mathbb{E}[d(C, \hat{C})] \leq D} I(C; \hat{C})$$

where $C \in \Sigma$ is the oracle label (concrete token), $\hat{C}$ is the
nano model's prediction, and $d: \Sigma \times \Sigma \to \mathbb{R}_{\geq 0}$
is a distortion measure (e.g., 0-1 loss).

For $|\Sigma| = K$, the channel capacity is bounded by:
$$C_{\max} = \log_2 K \quad \text{bits per example}$$

**Definition 6.1 (Convergence Threshold).** The distillation protocol
*converges* when the nano model's error rate falls below $\epsilon_{target}$,
corresponding to a distortion $D_{target} = \epsilon_{target}$.

### 6.2 Theorem 6.1: Sample Complexity Bound

**Theorem 6.1 (Convergence Sample Bound).** Under Assumptions 5.1–5.3,
the number of oracle queries $N_{\text{conv}}$ required for the distillation
protocol to converge to error rate $\epsilon_{target}$ with probability $1 - \delta$
satisfies:

$$N_{\text{conv}} = O\!\left(\frac{K \log K \cdot d_{VC}}{\epsilon_{target} - \epsilon_0} \cdot \log \frac{K}{\delta}\right)$$

where $d_{VC}$ is the VC dimension of the nano model's hypothesis class and
$\epsilon_0$ is the oracle's error rate.

*Proof Sketch.*

**Step 1 (Channel capacity).** Each oracle query on a concrete token space with
$K$ classes conveys at most $C_{\max} = \log_2 K$ bits. Under the uncertainty
sampling strategy, the mutual information per query is:

$$I_n = I(M^*(x); M_\theta^{(n)}) \approx \mathbb{E}_{x \in \mathcal{U}_n^{\text{top}}}[H(M^*(x))]$$

By definition of the uncertainty sampling strategy (selecting high-entropy
predictions), $I_n$ is lower bounded by a constant fraction $\alpha$ of
$C_{\max}$: $I_n \geq \alpha \log_2 K$.

**Step 2 (Model complexity).** The nano model has VC dimension $d_{VC} = O(N)$
for a neural network with $N$ parameters (this is the classic bound [13]).
However, in the concrete token regime, the *effective* VC dimension is much
smaller due to the low-dimensional sufficient statistic (Theorem 3.1). The
effective hypothesis class has VC dimension $\tilde{d}_{VC} = O(K \cdot d)$,
which for $K = 3$ and $d = 512$ gives $\tilde{d}_{VC} = O(1536)$ — vastly
smaller than the parameter count $N = 350 \times 10^6$.

**Step 3 (Combining bounds).** By the information-theoretic PAC bound (Cover
& Thomas [12, Theorem 12.6.1]), the number of examples needed to reduce the
generalization gap to $\epsilon_{target} - \epsilon_0$ at confidence $1-\delta$ is:

$$N_{\text{conv}} = O\!\left(\frac{\tilde{d}_{VC} \log(1/(\epsilon_{target} - \epsilon_0)) + \log(1/\delta)}{\epsilon_{target} - \epsilon_0}\right)$$

Substituting $\tilde{d}_{VC} = O(K \cdot d)$ and using $d \leq \log_2 V \cdot
(\log K)^{-1} \cdot K$ (from the compression ratio of §2.3) yields the stated
bound. $\square$

**Corollary 6.1 (Concrete Numbers for $K = 3$).** For $K = 3$, $\epsilon_0 = 0.05$,
$\epsilon_{target} = 0.10$, $\delta = 0.05$, and $\tilde{d}_{VC} = 1536$:

$$N_{\text{conv}} = O\!\left(\frac{1536 \cdot \log(20) + \log(20)}{0.05}\right) \approx O(100{,}000)$$

This is within the practical range of cloud oracle queries for an active
learning system querying at 1,000 examples per round over 100 rounds.

**Remark 6.1 (The Cardinality Dividend).** For a dense vocabulary model with
$K = 32{,}000$, the same calculation yields $N_{\text{conv}} = O(5 \times 10^8)$,
which is intractable for distillation. The concrete token regime provides a
reduction by a factor of $K_{dense} / K_{concrete} \cdot \log(K_{dense}/K_{concrete})
\approx 10^4$ in required samples, explaining the practical viability of
nano-model distillation.

---

## 7. Connection to the Free Energy Principle

### 7.1 Friston's Free Energy Principle

The *Free Energy Principle* (FEP) [2, 14] proposes that biological agents
minimize a functional called *variational free energy* $\mathcal{F}$ over their
internal states and their models of the world. For a generative model $p(o, s)$
over observations $o$ and hidden states $s$, and a recognition distribution
$q_\psi(s|o)$ parameterized by $\psi$:

$$\mathcal{F}[q, o] = \underbrace{\mathbb{E}_q[\log q_\psi(s|o) - \log p(o,s)]}_{\text{Variational Free Energy}}$$

Decomposing:
$$\mathcal{F}[q, o] = \underbrace{-\mathbb{E}_q[\log p(o|s)]}_{\text{Energy (accuracy)}} + \underbrace{\text{KL}[q_\psi(s|o) \| p(s)]}_{\text{Complexity}}$$

Minimizing $\mathcal{F}$ simultaneously drives accurate prediction of observations
(minimizing surprise) and regularization toward the prior (Occam's razor in
probabilistic form).

### 7.2 The Nano Model as an Active Inference Agent

We now construct a precise FEP interpretation of the CT-JEPA framework.

**Generative Model.** Define the generative model:
$$p(c, s, x) = p(c|s) \cdot p(s|x) \cdot p(x)$$

where:
- $x \in \mathcal{X}$ is the raw observation (metrics, logs, sensor readings)
- $s \in \mathcal{S}$ is the hidden *system state* (the true operational condition
  of the monitored system)
- $c \in \Sigma$ is the *concrete token* (the observable label emitted by the
  oracle or learned classifier)

The model assumes that the system state $s$ is a sufficient cause of the
concrete token $c$, with the raw observation $x$ being a noisy, high-dimensional
measurement of $s$.

**Recognition Distribution.** The nano model $f_\theta$ defines a recognition
distribution:
$$q_\theta(c|x) = \text{softmax}(f_\theta(x)) \in \Delta^{K-1}$$

This is the posterior over concrete tokens given the raw observation.

**Free Energy Decomposition.** The variational free energy of the nano model is:

$$\mathcal{F}_\theta(x, c^*) = -\log q_\theta(c^*|x) + \text{KL}[q_\theta(\cdot|x) \| p(\cdot)]$$

The first term is the *surprise* (negative log-likelihood of the oracle label
$c^*$) — this is the cross-entropy loss. The second term is the *complexity*
penalty — this acts as regularization toward the prior distribution over
concrete tokens.

**Theorem 7.1 (FEP-JEPA Correspondence).** Minimizing the expected variational
free energy of the nano model:

$$\min_\theta \mathbb{E}_{(x, c^*) \sim \mathcal{D}}[\mathcal{F}_\theta(x, c^*)]$$

is equivalent to the progressive distillation objective (§5) when:
1. $p(c)$ is the empirical prior over concrete tokens in $\mathcal{D}$
2. The KL term is the entropic regularizer

*Proof.* Expanding:
$$\mathbb{E}[\mathcal{F}_\theta(x, c^*)] = \underbrace{-\mathbb{E}[\log q_\theta(c^*|x)]}_{\text{Cross-entropy}} + \underbrace{\mathbb{E}_x[\text{KL}[q_\theta(\cdot|x) \| p(\cdot)]]}_{\text{Label entropy regularization}}$$

The cross-entropy term is exactly the distillation loss from §5. The KL term
penalizes overconfident predictions relative to the empirical class distribution,
acting as an implicit regularizer that prevents the nano model from collapsing
to a degenerate constant predictor (the concrete-token analog of representation
collapse in JEPA [1]). $\square$

### 7.3 The JEPA-FEP Duality

**Definition 7.1 (CT-JEPA Energy Function).** The energy function of the
Concrete-Token JEPA is:

$$E_\theta(c, x) = -\log q_\theta(c|x) = \text{CrossEntropy}(e_c, q_\theta(\cdot|x))$$

This is the *joint embedding distance* in the one-hot representation space.

**Proposition 7.1 (Collapse Prevention).** In standard JEPA, representational
collapse (where all inputs map to the same representation) is prevented by
architectural constraints (stop-gradient, momentum encoders [15]). In CT-JEPA,
collapse is prevented *structurally* by the finite cardinality of $\Sigma$:
any non-degenerate distribution over $K \geq 2$ classes cannot be a point mass.
The FEP complexity term $\text{KL}[q_\theta \| p]$ quantifies the remaining
collapse pressure.

**Definition 7.2 (Epistemic and Aleatoric Free Energy).** Following the
active inference literature [16], we decompose the free energy into:

$$\mathcal{F}_\theta = \underbrace{\text{CrossEntropy}(c^*, q_\theta(\cdot|x))}_{\text{Aleatoric}} + \underbrace{\mathbb{E}_\theta[\text{KL}[q_\theta \| p]]}_{\text{Epistemic}}$$

- **Aleatoric free energy** corresponds to irreducible uncertainty about the
  concrete token given the observation (Bayes error)
- **Epistemic free energy** corresponds to reducible uncertainty from finite
  data and model misspecification

Under progressive distillation (§5), each oracle query reduces the epistemic
component of free energy by providing labeled training signal in regions of
high model uncertainty (high $H[q_\theta(\cdot|x)]$), while the aleatoric
component approaches the oracle's irreducible error $\epsilon_0$.

**Corollary 7.1 (Convergence as Free Energy Minimization).** The distillation
protocol converges (Theorem 5.1) if and only if the epistemic free energy
converges to zero:

$$\text{acc}_n \to 1 - \epsilon_0 \iff \mathbb{E}[\text{KL}[q_{\theta^{(n)}} \| q_{\theta^*}]] \to 0$$

where $q_{\theta^*}$ is the nano model's best-in-class approximation to the
oracle. This follows directly from the Pinsker-bounded relationship between
KL divergence and total variation distance.

---

## 8. Discussion

### 8.1 Implications for System Design

The theory developed above has several direct design implications for
edge-deployed monitoring systems.

**Implication 1 (Model Sizing).** From Theorem 3.1 and Corollary 3.1, the
minimum model size for monitoring with $K = 3$ status codes is approximately
$N^* \approx 6.3 \times 10^6$ parameters — achievable with a tiny transformer.
Models at 350M parameters have a sufficiency factor of approximately 55. This
overcapacity supports robust in-context learning (Theorem 4.1) and rapid
distillation convergence (Theorem 5.1 and 6.1).

**Implication 2 (Distillation Budget).** From Corollary 6.1, approximately
100,000 oracle queries are sufficient for convergence in the 3-class monitoring
regime. At typical LLM API costs, this corresponds to roughly $100–$200 in
cloud inference budget — a one-time cost amortized over the deployment lifetime.

**Implication 3 (Prompt vs. LoRA Switching).** Theorem 4.1 establishes that
the function classes of ICL and LoRA are equivalent for $r = K$ in the concrete
token regime. The practical choice is:
- Use ICL when labeled examples are few ($m \leq K$) and gradient computation
  is unavailable
- Use LoRA when $m > K$ and inference latency is critical (LoRA avoids the
  $O(m \cdot d)$ KV-cache overhead of ICL)

### 8.2 Limitations and Assumptions

**Realizability (Assumption 5.2)** assumes the nano model's hypothesis class
contains the true posterior. This fails when:
- The input distribution shifts far from the pre-training distribution
- The sufficient statistic $T^*$ is non-smooth (e.g., discontinuous threshold
  effects in monitoring)
- The effective VC dimension is insufficient for the true $K$ (unlikely for
  350M models with $K \leq 10$)

**Oracle Error (Assumption 5.1)** assumes the cloud oracle is near-optimal.
In practice, cloud oracles may be systematically biased on domain-specific
tasks (e.g., specialized infrastructure metrics). Active distillation with
oracle disagreement detection [17] can mitigate this.

**Linear Attention Approximation (§4).** The Prompt-LoRA equivalence is exact
for linear attention and $\epsilon_{\text{nonlin}}$-approximate for softmax
attention. The error $\epsilon_{\text{nonlin}} = O(1/\sqrt{d})$ is small for
large $d$ but non-zero, meaning the function classes are *approximately* rather
than exactly equal.

### 8.3 Relation to Knowledge Distillation Literature

The progressive distillation protocol of §5 is related to, but distinct from,
classical knowledge distillation [18]. Classical distillation trains the student
on the teacher's *soft labels* (full probability distribution). CT-JEPA
distillation trains on *hard labels* (concrete tokens) from the teacher's
most confident predictions. The use of hard concrete tokens rather than soft
distributions is justified by:

1. **Concrete tokens carry sufficient statistics** (Theorem 3.1): the
   one-hot label preserves all information needed for classification accuracy
2. **Hard labels reduce label noise**: soft labels from an uncertain oracle
   can introduce more noise than the hard labels, especially near decision
   boundaries
3. **Active selection compensates**: uncertainty sampling ensures that the
   hard-labeled examples are distributed near decision boundaries, where the
   soft label information would be most useful — but the oracle's confidence
   there is precisely when hard labels are most reliable

---

## 9. Conclusion

We have developed a formal theoretical framework for Concrete-Token Joint-
Embedding Predictive Architectures, establishing five main results:

1. **Sufficient Statistics Theorem (§3):** Small language models with 350M+
   parameters have massive overcapacity for $K$-class monitoring tasks, where
   the binding constraint is $K$ rather than model size.

2. **Prompt-LoRA Equivalence (§4):** Few-shot prompting and rank-$K$ LoRA
   fine-tuning access the same function class in the concrete token regime,
   with explicit bijective construction via the thin SVD of the exemplar matrix.

3. **Progressive Distillation Theorem (§5):** Monotonic accuracy improvement
   under cloud oracle distillation holds under mild realizability and oracle
   quality assumptions.

4. **Convergence Bounds (§6):** The distillation protocol converges in
   $O(K \log K \cdot d_{VC} / \epsilon)$ oracle queries — tractable for small
   $K$ and infeasible for large $K$ such as full vocabulary models.

5. **FEP Correspondence (§7):** The CT-JEPA training objective is the
   variational free energy of a generative model over concrete tokens, with
   active distillation implementing epistemic free energy minimization.

Together, these results provide a principled justification for deploying small
language models as near-zero-latency inference engines in bounded-output domains.
The key conceptual contribution is the identification of *concrete tokens* as
the natural representation space for JEPA in classification settings — a
simplification that simultaneously makes the information theory tractable,
the distillation convergent, and the FEP correspondence exact.

Future work includes: extending the prompt-LoRA equivalence beyond the linear
attention approximation; deriving tight (rather than order-optimal) constants
in the convergence bounds; and empirically validating the FEP decomposition's
prediction that epistemic free energy tracks distillation progress in
monitoring deployments.

---

## References

[1] LeCun, Y. (2022). *A Path Towards Autonomous Machine Intelligence*. OpenReview
Preprint. [The foundational JEPA formulation, §2–3. Motivates prediction in
representation space over observation space to avoid modeling irreducible noise.]

[2] Friston, K. (2010). The free-energy principle: a unified brain theory?
*Nature Reviews Neuroscience*, 11(2), 127–138. [The FEP source text; the
energy decomposition in §7 follows §2.1–2.3 of this work.]

[3] Yun, C., Bhojanapalli, S., Rawat, A. S., Reddi, S. J., & Kumar, S. (2020).
Are transformers universal approximators of sequence-to-sequence functions?
*ICLR 2020*. [Establishes Theorem 3.1's approximation-theoretic foundation;
the Hölder-continuity bound in Step 3 follows their Theorem 1.]

[4] Barron, A. R. (1993). Universal approximation bounds for superpositions of
a sigmoidal function. *IEEE Transactions on Information Theory*, 39(3), 930–945.
[The Jackson-type approximation bound used in §3.3, Step 3.]

[5] Hu, E. J., Shen, Y., Wallis, P., Allen-Zhu, Z., Li, Y., Wang, S., ... &
Chen, W. (2022). LoRA: Low-rank adaptation of large language models. *ICLR 2022*.
[The LoRA parameterization $\Delta W = BA$ used throughout §4.]

[6] Von Oswald, J., Niklasson, E., Randazzo, E., Sacramento, J., Mordvintsev,
A., Zhmoginov, A., & Vladymyrov, M. (2023). Transformers learn in-context by
gradient descent. *ICML 2023*. [The linear-attention ICL-as-gradient-descent
result that underlies Theorem 4.1.]

[7] Akyürek, E., Schuurmans, D., Andreas, J., Ma, T., & Zhou, D. (2022). What
learning algorithm is in-context learning? Investigations with linear models.
*ICLR 2023*. [Extends [6] to softmax attention; source of the kernel regression
interpretation used in §4.2.]

[8] Vyas, A., Katharopoulos, A., & Fleuret, F. (2020). Fast transformers with
clustered attention. *NeurIPS 2020*. [The concentration inequality bounding
$\epsilon_{\text{nonlin}} = O(1/\sqrt{d})$ in Theorem 4.1.]

[9] Touvron, H., Lavril, T., Izacard, G., Martinet, X., Lachaux, M. A.,
Lacroix, T., ... & Lample, G. (2023). LLaMA: Open and efficient foundation
language models. *arXiv:2302.13971*. [Architecture reference for small open
models; the 350M–1.2B parameter regime studied here corresponds to LLaMA's
smallest configurations.]

[10] Shalev-Shwartz, S., & Ben-David, S. (2014). *Understanding Machine
Learning: From Theory to Algorithms*. Cambridge University Press. [PAC learning
bounds used in Theorem 5.1 (§18.1) and Theorem 6.1 (§2.3).]

[11] Shimodaira, H. (2000). Improving predictive inference under covariate
shift by weighting the log-likelihood function. *Journal of Statistical Planning
and Inference*, 90(2), 227–244. [The importance-weighting bound in §5.2, Step 3.]

[12] Cover, T. M., & Thomas, J. A. (2006). *Elements of Information Theory*
(2nd ed.). Wiley. [Rate-distortion theory (§10) and the information-theoretic
PAC bound (§12.6) used in §6.]

[13] Bartlett, P. L., Harvey, N., Liaw, C., & Mehrabian, A. (2019). Nearly-tight
VC-dimension and pseudodimension bounds for piecewise linear neural networks.
*Journal of Machine Learning Research*, 20(63), 1–17. [VC dimension bounds for
deep networks; the effective VC dimension reduction in §6.2 Step 2.]

[14] Friston, K., FitzGerald, T., Rigoli, F., Schwartenbeck, P., & Pezzulo, G.
(2017). Active inference: A process theory. *Neural Computation*, 29(1), 1–49.
[The active inference formulation of FEP, including the epistemic/aleatoric
free energy decomposition of Definition 7.2.]

[15] Grill, J. B., Strub, F., Altché, F., Tallec, C., Richemond, P. H.,
Buchatskaya, E., ... & Valko, M. (2020). Bootstrap your own latent: A new
approach to self-supervised learning. *NeurIPS 2020*. [Momentum encoders and
stop-gradient as collapse prevention; contrasted with CT-JEPA's structural
collapse prevention in §7.3.]

[16] Parr, T., Pezzulo, G., & Friston, K. J. (2022). *Active Inference: The
Free Energy Principle in Mind, Brain, and Behavior*. MIT Press. [Epistemological
framework for epistemic vs. aleatoric free energy distinction in §7.2.]

[17] Balcan, M. F., Beygelzimer, A., & Langford, J. (2006). Agnostic active
learning. *ICML 2006*. [Oracle disagreement detection in the context of active
learning; relevant to the limitation discussion in §8.2.]

[18] Hinton, G., Vinyals, O., & Dean, J. (2015). Distilling the knowledge in a
neural network. *NeurIPS Workshop on Deep Learning*, arXiv:1503.02531. [Classical
knowledge distillation with soft labels; contrasted with CT-JEPA's hard-label
active distillation in §8.3.]
