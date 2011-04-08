/*
 *  FPGraphics.m
 *  IronJump
 *
 *  Created by Filip Kunc on 5/30/10.
 *  For license see LICENSE.TXT
 *
 */

#import "FPGraphics.h"

FPAtlasVertex globalVertexBuffer[kMaxVertices];
float rectTolerance = 0.1f;

CGRect CGRectMakeFromPoints(CGPoint a, CGPoint b)
{
	CGFloat x1 = MIN(a.x, b.x);
	CGFloat y1 = MIN(a.y, b.y);
	CGFloat x2 = MAX(a.x, b.x);
	CGFloat y2 = MAX(a.y, b.y);
	
	return CGRectMake(x1, y1, x2 - x1, y2 - y1);
}

CGPoint CGRectMiddlePoint(CGRect rect)
{
	return CGPointMake(CGRectGetMidX(rect), CGRectGetMidY(rect));
}

CGRect CGRectWithMove(CGRect rect, float moveX, float moveY)
{
	if (moveX < 0.0f)
	{
		rect.origin.x += moveX;
		rect.size.width -= moveX;
	}
	else
	{
		rect.size.width += moveX;
	}
	
	if (moveY < 0.0f)
	{
		rect.origin.y += moveY;
		rect.size.height -= moveY;
	}
	else
	{
		rect.size.height += moveY;
	}
	
	return rect;
}

BOOL CGRectIntersectsRectWithTolerance(CGRect a, CGRect b)
{
    CGRect intersection = CGRectIntersection(a, b);
    if (CGRectIsEmptyWithTolerance(intersection))
        return NO;
    return YES;
}

BOOL CGRectIsEmptyWithTolerance(CGRect a)
{
    if (a.size.width < rectTolerance || a.size.height < rectTolerance)
        return YES;
    return NO;
}
