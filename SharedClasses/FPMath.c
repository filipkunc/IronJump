/*
 *  FPMath.c
 *  IronJump
 *
 *  Created by Filip Kunc on 5/5/10.
 *  For license see LICENSE.TXT
 *
 */

#include "FPMath.h"

float fabsmaxf(float n, float max)
{
	return n > 0.0f ? fmaxf(n, max) : fminf(n, -max);
}

float fabsminf(float n, float min)
{
	return n > 0.0f ? fminf(n, min) : fmaxf(n, -min);
}

float flerpf(float a, float b, float w)
{
	return a * (1.0f - w) + b * w;
}